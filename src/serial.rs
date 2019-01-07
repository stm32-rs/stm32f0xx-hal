//! API for the integrated USART ports
//!
//! This only implements the usual asynchronous bidirectional 8-bit transfers, everything else is missing
//!
//! # Examples
//! ``` no_run
//! use stm32f0xx_hal as hal;
//!
//! use crate::hal::prelude::*;
//! use crate::hal::serial::Serial;
//! use crate::hal::stm32;
//!
//! use nb::block;
//!
//! cortex_m::interrupt::free(|cs| {
//!     let rcc = p.RCC.configure().sysclk(48.mhz()).freeze();
//!
//!     let gpioa = p.GPIOA.split(&mut rcc);
//!
//!     let tx = gpioa.pa9.into_alternate_af1(cs);
//!     let rx = gpioa.pa10.into_alternate_af1(cs);
//!
//!     let serial = Serial::usart1(p.USART1, (tx, rx), 115_200.bps(), &mut rcc);
//!
//!     let (mut tx, mut rx) = serial.split();
//!
//!     loop {
//!         let received = block!(rx.read()).unwrap();
//!         block!(tx.write(received)).ok();
//!     }
//! });
//! ```

use core::{
    fmt::{Result, Write},
    ops::Deref,
    ptr,
};

use embedded_hal::prelude::*;

use crate::{gpio::*, rcc::Rcc, time::Bps};

use core::marker::PhantomData;

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

/// Interrupt event
pub enum Event {
    /// New data has been received
    Rxne,
    /// New data can be sent
    Txe,
    /// Idle line state detected
    Idle,
}

pub trait TxPin<USART> {}
pub trait RxPin<USART> {}

macro_rules! usart_pins {
    ($($USART:ident => {
        tx => [$($tx:ty),+ $(,)*],
        rx => [$($rx:ty),+ $(,)*],
    })+) => {
        $(
            $(
                impl TxPin<crate::stm32::$USART> for $tx {}
            )+
            $(
                impl RxPin<crate::stm32::$USART> for $rx {}
            )+
        )+
    }
}

#[cfg(any(
    feature = "stm32f030",
    feature = "stm32f042",
    feature = "stm32f051",
    feature = "stm32f071",
))]
usart_pins! {
    USART1 =>  {
        tx => [gpioa::PA9<Alternate<AF1>>, gpiob::PB6<Alternate<AF0>>],
        rx => [gpioa::PA10<Alternate<AF1>>, gpiob::PB6<Alternate<AF0>>],
    }
}
#[cfg(any(feature = "stm32f031", feature = "stm32f030x6"))]
usart_pins! {
    USART1 => {
        tx => [gpioa::PA2<Alternate<AF1>>, gpioa::PA14<Alternate<AF1>>],
        rx => [gpioa::PA3<Alternate<AF1>>, gpioa::PA15<Alternate<AF1>>],
    }
}
#[cfg(any(
    feature = "stm32f031",
    feature = "stm32f070",
    feature = "stm32f072",
    feature = "stm32f091",
))]
usart_pins! {
    USART1 => {
        tx => [gpioa::PA9<Alternate<AF1>>, gpiob::PB6<Alternate<AF0>>],
        rx => [gpioa::PA10<Alternate<AF1>>, gpiob::PB7<Alternate<AF0>>],
    }
}

#[cfg(any(
    feature = "stm32f030x8",
    feature = "stm32f030xc",
    feature = "stm32f042",
    feature = "stm32f051",
    feature = "stm32f070",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f091",
))]
usart_pins! {
    USART2 => {
        tx => [gpioa::PA2<Alternate<AF1>>, gpioa::PA14<Alternate<AF1>>],
        rx => [gpioa::PA3<Alternate<AF1>>, gpioa::PA15<Alternate<AF1>>],
    }
}
#[cfg(any(feature = "stm32f072", feature = "stm32f071", feature = "stm32f091"))]
usart_pins! {
    USART2 => {
        tx => [gpiod::PD5<Alternate<AF0>>],
        rx => [gpiod::PD6<Alternate<AF0>>],
    }
}

#[cfg(any(
    feature = "stm32f030xc",
    feature = "stm32f070xb",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f091",
))]
usart_pins! {
    USART3 => {
        // According to the datasheet PB10 is both tx and rx, but in stm32cubemx it's only tx
        tx => [gpiob::PB10<Alternate<AF4>>, gpioc::PC4<Alternate<AF1>>, gpioc::PC10<Alternate<AF1>>],
        rx => [gpiob::PB11<Alternate<AF4>>, gpioc::PC5<Alternate<AF1>>, gpioc::PC11<Alternate<AF1>>],
    }
    USART4 => {
        tx => [gpioa::PA0<Alternate<AF4>>, gpioc::PC10<Alternate<AF0>>],
        rx => [gpioa::PA1<Alternate<AF4>>, gpioc::PC11<Alternate<AF0>>],
    }
}
#[cfg(any(feature = "stm32f071", feature = "stm32f072", feature = "stm32f091"))]
usart_pins! {
    USART3 => {
        tx => [gpiod::PD8<Alternate<AF0>>],
        rx => [gpiod::PD9<Alternate<AF0>>],
    }
}

// TODO: The ST SVD files are missing the entire PE enable register.
//       Re-enable as soon as this gets fixed.
// #[cfg(feature = "stm32f091")]
// usart_pins! {
//     USART4 => {
//         tx => [gpioe::PE8<Alternate<AF1>>],
//         rx => [gpioe::PE9<Alternate<AF1>>],
//     }
// }

#[cfg(any(feature = "stm32f030xc", feature = "stm32f091"))]
usart_pins! {
    USART5 => {
        tx => [gpiob::PB3<Alternate<AF4>>, gpioc::PC12<Alternate<AF2>>],
        rx => [gpiob::PB4<Alternate<AF4>>, gpiod::PD2<Alternate<AF2>>],
    }
    USART6 => {
        tx => [gpioa::PA4<Alternate<AF5>>, gpioc::PC0<Alternate<AF2>>],
        rx => [gpioa::PA5<Alternate<AF5>>, gpioc::PC1<Alternate<AF2>>],
    }
}
// TODO: The ST SVD files are missing the entire PE enable register.
//       Re-enable as soon as this gets fixed.
#[cfg(feature = "stm32f091")]
usart_pins! {
    // USART5 => {
    //     tx => [gpioe::PE10<Alternate<AF1>>],
    //     rx => [gpioe::PE11<Alternate<AF1>>],
    // }
    USART6 => {
        tx => [gpiof::PF9<Alternate<AF1>>],
        rx => [gpiof::PF10<Alternate<AF1>>],
    }
}

/// Serial abstraction
pub struct Serial<USART, TXPIN, RXPIN> {
    usart: USART,
    pins: (TXPIN, RXPIN),
}

// Common register
type SerialRegisterBlock = crate::stm32::usart1::RegisterBlock;

/// Serial receiver
pub struct Rx<USART> {
    usart: *const SerialRegisterBlock,
    _instance: PhantomData<USART>,
}

// NOTE(unsafe) Required to allow protected shared access in handlers
unsafe impl<USART> Send for Rx<USART> {}

/// Serial transmitter
pub struct Tx<USART> {
    usart: *const SerialRegisterBlock,
    _instance: PhantomData<USART>,
}

// NOTE(unsafe) Required to allow protected shared access in handlers
unsafe impl<USART> Send for Tx<USART> {}

macro_rules! usart {
    ($($USART:ident: ($usart:ident, $usarttx:ident, $usartrx:ident, $usartXen:ident, $apbenr:ident),)+) => {
        $(
            use crate::stm32::$USART;
            impl<TXPIN, RXPIN> Serial<$USART, TXPIN, RXPIN>
            where
                TXPIN: TxPin<$USART>,
                RXPIN: RxPin<$USART>,
            {
                /// Creates a new serial instance
                pub fn $usart(usart: $USART, pins: (TXPIN, RXPIN), baud_rate: Bps, rcc: &mut Rcc) -> Self
                {
                    let mut serial = Serial { usart, pins };
                    serial.configure(baud_rate, rcc);
                    // Enable transmission and receiving
                    serial.usart.cr1.modify(|_, w| w.te().set_bit().re().set_bit().ue().set_bit());
                    serial
                }
            }

            impl<TXPIN> Serial<$USART, TXPIN, ()>
            where
                TXPIN: TxPin<$USART>,
            {
                /// Creates a new tx-only serial instance
                pub fn $usarttx(usart: $USART, txpin: TXPIN, baud_rate: Bps, rcc: &mut Rcc) -> Self
                {
                    let rxpin = ();
                    let mut serial = Serial { usart, pins: (txpin, rxpin) };
                    serial.configure(baud_rate, rcc);
                    // Enable transmission
                    serial.usart.cr1.modify(|_, w| w.te().set_bit().ue().set_bit());
                    serial
                }
            }

            impl<RXPIN> Serial<$USART, (), RXPIN>
            where
                RXPIN: RxPin<$USART>,
            {
                /// Creates a new tx-only serial instance
                pub fn $usartrx(usart: $USART, rxpin: RXPIN, baud_rate: Bps, rcc: &mut Rcc) -> Self
                {
                    let txpin = ();
                    let mut serial = Serial { usart, pins: (txpin, rxpin) };
                    serial.configure(baud_rate, rcc);
                    // Enable receiving
                    serial.usart.cr1.modify(|_, w| w.re().set_bit().ue().set_bit());
                    serial
                }
            }

            impl<TXPIN, RXPIN> Serial<$USART, TXPIN, RXPIN> {
                fn configure(&mut self, baud_rate: Bps, rcc: &mut Rcc) {
                    // Enable clock for USART
                    rcc.regs.$apbenr.modify(|_, w| w.$usartXen().set_bit());

                    // Calculate correct baudrate divisor on the fly
                    let brr = rcc.clocks.pclk().0 / baud_rate.0;
                    self.usart.brr.write(|w| unsafe { w.bits(brr) });

                    // Reset other registers to disable advanced USART features
                    self.usart.cr2.reset();
                    self.usart.cr3.reset();
                }

                /// Starts listening for an interrupt event
                pub fn listen(&mut self, event: Event) {
                    match event {
                        Event::Rxne => {
                            self.usart.cr1.modify(|_, w| w.rxneie().set_bit())
                        },
                        Event::Txe => {
                            self.usart.cr1.modify(|_, w| w.txeie().set_bit())
                        },
                        Event::Idle => {
                            self.usart.cr1.modify(|_, w| w.idleie().set_bit())
                        },
                    }
                }

                /// Stop listening for an interrupt event
                pub fn unlisten(&mut self, event: Event) {
                    match event {
                        Event::Rxne => {
                            self.usart.cr1.modify(|_, w| w.rxneie().clear_bit())
                        },
                        Event::Txe => {
                            self.usart.cr1.modify(|_, w| w.txeie().clear_bit())
                        },
                        Event::Idle => {
                            self.usart.cr1.modify(|_, w| w.idleie().clear_bit())
                        },
                    }
                }
            }
        )+
    }
}

usart! {
    USART1: (usart1, usart1tx, usart1rx, usart1en, apb2enr),
}
#[cfg(any(
    feature = "stm32f030x8",
    feature = "stm32f030xc",
    feature = "stm32f042",
    feature = "stm32f051",
    feature = "stm32f070",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f091",
))]
usart! {
    USART2: (usart2, usart2tx, usart2rx,usart2en, apb1enr),
}
#[cfg(any(
    feature = "stm32f030xc",
    feature = "stm32f070xb",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f091",
))]
usart! {
    USART3: (usart3, usart3tx, usart3rx,usart3en, apb1enr),
    USART4: (usart4, usart4tx, usart4rx,usart4en, apb1enr),
}
#[cfg(any(feature = "stm32f030xc", feature = "stm32f091"))]
usart! {
    USART5: (usart5, usart5tx, usart5rx,usart5en, apb1enr),
    USART6: (usart6, usart6tx, usart6rx,usart6en, apb2enr),
}

impl<USART> embedded_hal::serial::Read<u8> for Rx<USART>
where
    USART: Deref<Target = SerialRegisterBlock>,
{
    type Error = Error;

    /// Tries to read a byte from the uart
    fn read(&mut self) -> nb::Result<u8, Error> {
        read(self.usart)
    }
}

impl<USART, TXPIN, RXPIN> embedded_hal::serial::Read<u8> for Serial<USART, TXPIN, RXPIN>
where
    USART: Deref<Target = SerialRegisterBlock>,
    RXPIN: RxPin<USART>,
{
    type Error = Error;

    /// Tries to read a byte from the uart
    fn read(&mut self) -> nb::Result<u8, Error> {
        read(&*self.usart)
    }
}

impl<USART> embedded_hal::serial::Write<u8> for Tx<USART>
where
    USART: Deref<Target = SerialRegisterBlock>,
{
    type Error = void::Void;

    /// Ensures that none of the previously written words are still buffered
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        flush(self.usart)
    }

    /// Tries to write a byte to the uart
    /// Fails if the transmit buffer is full
    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        write(self.usart, byte)
    }
}

impl<USART, TXPIN, RXPIN> embedded_hal::serial::Write<u8> for Serial<USART, TXPIN, RXPIN>
where
    USART: Deref<Target = SerialRegisterBlock>,
    TXPIN: TxPin<USART>,
{
    type Error = void::Void;

    /// Ensures that none of the previously written words are still buffered
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        flush(&*self.usart)
    }

    /// Tries to write a byte to the uart
    /// Fails if the transmit buffer is full
    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        write(&*self.usart, byte)
    }
}

impl<USART, TXPIN, RXPIN> Serial<USART, TXPIN, RXPIN>
where
    USART: Deref<Target = SerialRegisterBlock>,
{
    /// Splits the UART Peripheral in a Tx and an Rx part
    /// This is required for sending/receiving
    pub fn split(self) -> (Tx<USART>, Rx<USART>)
    where
        TXPIN: TxPin<USART>,
        RXPIN: RxPin<USART>,
    {
        (
            Tx {
                usart: &*self.usart,
                _instance: PhantomData,
            },
            Rx {
                usart: &*self.usart,
                _instance: PhantomData,
            },
        )
    }

    pub fn release(self) -> (USART, (TXPIN, RXPIN)) {
        (self.usart, self.pins)
    }
}

impl<USART> Write for Tx<USART>
where
    Tx<USART>: embedded_hal::serial::Write<u8>,
{
    fn write_str(&mut self, s: &str) -> Result {
        use nb::block;

        let _ = s.as_bytes().iter().map(|c| block!(self.write(*c))).last();
        Ok(())
    }
}

/// Ensures that none of the previously written words are still buffered
fn flush(usart: *const SerialRegisterBlock) -> nb::Result<(), void::Void> {
    // NOTE(unsafe) atomic read with no side effects
    let isr = unsafe { (*usart).isr.read() };

    if isr.tc().bit_is_set() {
        Ok(())
    } else {
        Err(nb::Error::WouldBlock)
    }
}

/// Tries to write a byte to the uart
/// Fails if the transmit buffer is full
fn write(usart: *const SerialRegisterBlock, byte: u8) -> nb::Result<(), void::Void> {
    // NOTE(unsafe) atomic read with no side effects
    let isr = unsafe { (*usart).isr.read() };

    if isr.txe().bit_is_set() {
        // NOTE(unsafe) atomic write to stateless register
        // NOTE(write_volatile) 8-bit write that's not possible through the svd2rust API
        unsafe { ptr::write_volatile(&(*usart).tdr as *const _ as *mut _, byte) }
        Ok(())
    } else {
        Err(nb::Error::WouldBlock)
    }
}

/// Tries to read a byte from the uart
fn read(usart: *const SerialRegisterBlock) -> nb::Result<u8, Error> {
    // NOTE(unsafe) atomic read with no side effects
    let isr = unsafe { (*usart).isr.read() };

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
        return Ok(unsafe { ptr::read_volatile(&(*usart).rdr as *const _ as *const _) });
    } else {
        nb::Error::WouldBlock
    })
}
