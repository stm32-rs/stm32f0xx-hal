//! API for the integrated USART ports
//!
//! This only implements the usual asynchronous bidirectional 8-bit transfers, everything else is missing
//!
//! # Example
//! Serial Echo
//! ``` no_run
//! use stm32f0xx_hal as hal;
//!
//! use crate::hal::stm32;
//! use crate::hal::prelude::*;
//! use crate::hal::serial::Serial;
//! use nb::block;
//!
//! let mut p = stm32::Peripherals::take().unwrap();
//!
//! let mut led = gpioa.pa1.into_push_pull_pull_output();
//! let rcc = p.RCC.constrain().cfgr.freeze();
//! let mut timer = Timer::tim1(p.TIM1, Hertz(1), clocks);
//! loop {
//!     led.toggle();
//!     block!(timer.wait()).ok();
//! }
//! ```

use core::fmt::{Result, Write};
use core::ops::Deref;
use core::ptr;

use embedded_hal::prelude::*;
use nb::block;
use void::Void;

#[cfg(any(feature = "stm32f042", feature = "stm32f030"))]
use crate::stm32;

use crate::gpio::*;
use crate::rcc::Clocks;
use crate::time::Bps;

/// Interrupt event
pub enum Event {
    /// New data has been received
    Rxne,
    /// New data can be sent
    Txe,
}

/// Serial error
#[derive(Debug)]
pub enum Error {
    /// Framing error
    Framing,
    /// Noise error
    Noise,
    /// RX buffer overrun
    Overrun,
    /// Parity check error
    Parity,
    #[doc(hidden)]
    _Extensible,
}

pub trait Pins<USART> {}

#[cfg(any(feature = "stm32f030", feature = "stm32f042"))]
impl Pins<stm32::USART1> for (gpioa::PA9<Alternate<AF1>>, gpioa::PA10<Alternate<AF1>>) {}
#[cfg(any(feature = "stm32f030", feature = "stm32f042"))]
impl Pins<stm32::USART1> for (gpiob::PB6<Alternate<AF0>>, gpiob::PB7<Alternate<AF0>>) {}
#[cfg(any(feature = "stm32f030", feature = "stm32f042"))]
impl Pins<stm32::USART1> for (gpioa::PA9<Alternate<AF1>>, gpiob::PB7<Alternate<AF0>>) {}
#[cfg(any(feature = "stm32f030", feature = "stm32f042"))]
impl Pins<stm32::USART1> for (gpiob::PB6<Alternate<AF0>>, gpioa::PA10<Alternate<AF1>>) {}

#[cfg(feature = "stm32f030x6")]
impl Pins<stm32::USART1> for (gpioa::PA2<Alternate<AF1>>, gpioa::PA3<Alternate<AF1>>) {}

#[cfg(any(
    feature = "stm32f042",
    feature = "stm32f030x8",
    feature = "stm32f030xc",
))]
impl Pins<stm32::USART2> for (gpioa::PA2<Alternate<AF1>>, gpioa::PA3<Alternate<AF1>>) {}
#[cfg(any(
    feature = "stm32f042",
    feature = "stm32f030x8",
    feature = "stm32f030xc",
))]
impl Pins<stm32::USART2> for (gpioa::PA2<Alternate<AF1>>, gpioa::PA15<Alternate<AF1>>) {}
#[cfg(any(
    feature = "stm32f042",
    feature = "stm32f030x8",
    feature = "stm32f030xc",
))]
impl Pins<stm32::USART2> for (gpioa::PA14<Alternate<AF1>>, gpioa::PA15<Alternate<AF1>>) {}
#[cfg(any(
    feature = "stm32f042",
    feature = "stm32f030x8",
    feature = "stm32f030xc",
))]
impl Pins<stm32::USART2> for (gpioa::PA14<Alternate<AF1>>, gpioa::PA3<Alternate<AF1>>) {}

/// Serial abstraction
pub struct Serial<USART, PINS> {
    usart: USART,
    pins: PINS,
}

/// Serial receiver
pub struct Rx<USART> {
    // This is ok, because the USART types only contains PhantomData
    usart: *const USART,
}

/// Serial transmitter
pub struct Tx<USART> {
    // This is ok, because the USART types only contains PhantomData
    usart: *const USART,
}

macro_rules! usart {
    ($($USART:ident: ($usart:ident, $usartXen:ident, $apbenr:ident),)+) => {
        $(
            use crate::stm32::$USART;
            impl<PINS> Serial<$USART, PINS> {
                /// Creates a new serial instance
                pub fn $usart(usart: $USART, pins: PINS, baud_rate: Bps, clocks: Clocks) -> Self
                where
                    PINS: Pins<$USART>,
                {
                    // NOTE(unsafe) This executes only during initialisation
                    let rcc = unsafe { &(*stm32::RCC::ptr()) };

                    /* Enable clock for USART */
                    rcc.$apbenr.modify(|_, w| w.$usartXen().set_bit());

                    // Calculate correct baudrate divisor on the fly
                    let brr = clocks.pclk().0 / baud_rate.0;
                    usart.brr.write(|w| unsafe { w.bits(brr) });

                    /* Reset other registers to disable advanced USART features */
                    usart.cr2.reset();
                    usart.cr3.reset();

                    /* Enable transmission and receiving */
                    usart.cr1.modify(|_, w| unsafe { w.bits(0xD) });

                    Serial { usart, pins }
                }
            }
        )+
    }
}

#[cfg(any(feature = "stm32f042", feature = "stm32f030"))]
usart! {
    USART1: (usart1, usart1en, apb2enr),
}
#[cfg(any(
    feature = "stm32f042",
    feature = "stm32f030x8",
    feature = "stm32f030xc"
))]
usart! {
    USART2: (usart2, usart2en, apb1enr),
}
#[cfg(any(feature = "stm32f030xc"))]
usart! {
    USART3: (usart3, usart3en, apb1enr),
    USART4: (usart4, usart4en, apb1enr),
    USART5: (usart5, usart5en, apb1enr),
    // the usart6en bit is missing
    // USART6: (usart6, usart6en, apb2enr),
}

// It's s needed for the impls, but rustc doesn't recognize that
#[allow(dead_code)]
type SerialRegisterBlock = stm32::usart1::RegisterBlock;

impl<USART> embedded_hal::serial::Read<u8> for Rx<USART>
where
    USART: Deref<Target = SerialRegisterBlock>,
{
    type Error = Error;

    /// Tries to read a byte from the uart
    fn read(&mut self) -> nb::Result<u8, Error> {
        // NOTE(unsafe) atomic read with no side effects
        let isr = unsafe { (*self.usart).isr.read() };

        Err(if isr.pe().bit_is_set() {
            nb::Error::Other(Error::Parity)
        } else if isr.fe().bit_is_set() {
            nb::Error::Other(Error::Framing)
        } else if isr.nf().bit_is_set() {
            nb::Error::Other(Error::Noise)
        } else if isr.ore().bit_is_set() {
            nb::Error::Other(Error::Overrun)
        } else if isr.rxne().bit_is_set() {
            // NOTE(read_volatile) see `write_volatile` below
            return Ok(unsafe { ptr::read_volatile(&(*self.usart).rdr as *const _ as *const _) });
        } else {
            nb::Error::WouldBlock
        })
    }
}

impl<USART> embedded_hal::serial::Write<u8> for Tx<USART>
where
    USART: Deref<Target = SerialRegisterBlock>,
{
    type Error = Void;

    /// Ensures that none of the previously written words are still buffered
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        // NOTE(unsafe) atomic read with no side effects
        let isr = unsafe { (*self.usart).isr.read() };

        if isr.tc().bit_is_set() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    /// Tries to write a byte to the uart
    /// Fails if the transmit buffer is full
    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        // NOTE(unsafe) atomic read with no side effects
        let isr = unsafe { (*self.usart).isr.read() };

        if isr.txe().bit_is_set() {
            // NOTE(unsafe) atomic write to stateless register
            // NOTE(write_volatile) 8-bit write that's not possible through the svd2rust API
            unsafe { ptr::write_volatile(&(*self.usart).tdr as *const _ as *mut _, byte) }
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<USART, PINS> Serial<USART, PINS>
where
    USART: Deref<Target = SerialRegisterBlock>,
    PINS: Pins<USART>,
{
    /// Splits the UART Peripheral in a Tx and an Rx part
    /// This is required for sending/receiving
    pub fn split(self) -> (Tx<USART>, Rx<USART>) {
        (
            Tx {
                usart: &self.usart as *const _,
            },
            Rx {
                usart: &self.usart as *const _,
            },
        )
    }
    pub fn release(self) -> (USART, PINS) {
        (self.usart, self.pins)
    }
}

impl<USART> Write for Tx<USART>
where
    Tx<USART>: embedded_hal::serial::Write<u8>,
{
    fn write_str(&mut self, s: &str) -> Result {
        let _ = s.as_bytes().iter().map(|c| block!(self.write(*c))).last();
        Ok(())
    }
}
