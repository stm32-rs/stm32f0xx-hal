//! API for the integrate SPI peripherals
//!
//! The spi bus acts as the master (generating the clock) and you need to handle the CS separately.
//!
//! The most significant bit is transmitted first & only 8-bit transfers are supported
//!
//! # Example
//! Echo incoming data in the next transfer
//! ``` no_run
//! use stm32f0xx_hal as hal;
//!
//! use crate::hal::stm32;
//! use crate::hal::prelude::*;
//! use crate::hal::spi::{Spi, Mode, Phase, Polarity};
//!
//! cortex_m::interrupt::free(|cs| {
//!     let mut p = stm32::Peripherals::take().unwrap();
//!     let mut rcc = p.RCC.constrain().freeze(&mut p.FLASH);
//!
//!     let gpioa = p.GPIOA.split(&mut rcc);
//!
//!     // Configure pins for SPI
//!     let sck = gpioa.pa5.into_alternate_af0(cs);
//!     let miso = gpioa.pa6.into_alternate_af0(cs);
//!     let mosi = gpioa.pa7.into_alternate_af0()cs;
//!
//!     // Configure SPI with 1MHz rate
//!     let mut spi = Spi::spi1(p.SPI1, (sck, miso, mosi), Mode {
//!         polarity: Polarity::IdleHigh,
//!         phase: Phase::CaptureOnSecondTransition,
//!     }, 1.mhz(), &mut rcc);
//!
//!     let mut data = [ 0 ];
//!     loop {
//!         spi.transfer(&mut data).unwrap();
//!     }
//! });
//! ```

use core::{ops::Deref, ptr};

use nb;

pub use embedded_hal::spi::{Mode, Phase, Polarity};

// TODO Put this inside the macro
// Currently that causes a compiler panic
use crate::stm32::SPI1;
#[cfg(any(
    feature = "stm32f030x8",
    feature = "stm32f030xc",
    feature = "stm32f042",
    feature = "stm32f048",
    feature = "stm32f051",
    feature = "stm32f058",
    feature = "stm32f070xb",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
use crate::stm32::SPI2;

use crate::gpio::*;

use crate::rcc::{Clocks, Rcc};

use crate::time::Hertz;

/// SPI error
#[derive(Debug)]
pub enum Error {
    /// Overrun occurred
    Overrun,
    /// Mode fault occurred
    ModeFault,
    /// CRC error
    Crc,
    #[doc(hidden)]
    _Extensible,
}

/// SPI abstraction
pub struct Spi<SPI, SCKPIN, MISOPIN, MOSIPIN> {
    spi: SPI,
    pins: (SCKPIN, MISOPIN, MOSIPIN),
}

pub trait SckPin<SPI> {}
pub trait MisoPin<SPI> {}
pub trait MosiPin<SPI> {}

macro_rules! spi_pins {
    ($($SPI:ident => {
        sck => [$($sck:ty),+ $(,)*],
        miso => [$($miso:ty),+ $(,)*],
        mosi => [$($mosi:ty),+ $(,)*],
    })+) => {
        $(
            $(
                impl SckPin<crate::stm32::$SPI> for $sck {}
            )+
            $(
                impl MisoPin<crate::stm32::$SPI> for $miso {}
            )+
            $(
                impl MosiPin<crate::stm32::$SPI> for $mosi {}
            )+
        )+
    }
}

spi_pins! {
    SPI1 => {
        sck => [gpioa::PA5<Alternate<AF0>>, gpiob::PB3<Alternate<AF0>>],
        miso => [gpioa::PA6<Alternate<AF0>>, gpiob::PB4<Alternate<AF0>>],
        mosi => [gpioa::PA7<Alternate<AF0>>, gpiob::PB5<Alternate<AF0>>],
    }
}
#[cfg(any(
    feature = "stm32f030x4",
    feature = "stm32f030x6",
    feature = "stm32f031",
    feature = "stm32f038",
))]
spi_pins! {
    SPI1 => {
        sck => [gpiob::PB13<Alternate<AF0>>],
        miso => [gpiob::PB14<Alternate<AF0>>],
        mosi => [gpiob::PB15<Alternate<AF0>>],
    }
}
// TODO: The ST SVD files are missing the entire PE enable register.
//       So those pins do not exist in the register definitions.
//       Re-enable as soon as this gets fixed.
// #[cfg(any(
//     feature = "stm32f071",
//     feature = "stm32f072",
//     feature = "stm32f078",
//     feature = "stm32f091",
//     feature = "stm32f098",
// ))]
// spi_pins! {
//     SPI1 => {
//         sck => [gpioe::PE13<Alternate<AF1>>],
//         miso => [gpioe::PE14<Alternate<AF1>>],
//         mosi => [gpioe::PE15<Alternate<AF1>>],
//     }
// }

#[cfg(any(
    feature = "stm32f030x8",
    feature = "stm32f030xc",
    feature = "stm32f042",
    feature = "stm32f048",
    feature = "stm32f051",
    feature = "stm32f058",
    feature = "stm32f070xb",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
spi_pins! {
    SPI2 => {
        sck => [gpiob::PB13<Alternate<AF0>>],
        miso => [gpiob::PB14<Alternate<AF0>>],
        mosi => [gpiob::PB15<Alternate<AF0>>],
    }
}
#[cfg(any(
    feature = "stm32f030xc",
    feature = "stm32f070xb",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
spi_pins! {
    SPI2 => {
        sck => [gpiob::PB10<Alternate<AF5>>],
        miso => [gpioc::PC2<Alternate<AF1>>],
        mosi => [gpioc::PC3<Alternate<AF1>>],
    }
}
#[cfg(any(
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
spi_pins! {
    SPI2 => {
        sck => [gpiod::PD1<Alternate<AF1>>],
        miso => [gpiod::PD3<Alternate<AF1>>],
        mosi => [gpiod::PD4<Alternate<AF1>>],
    }
}

macro_rules! spi {
    ($($SPI:ident: ($spi:ident, $spiXen:ident, $spiXrst:ident, $apbenr:ident, $apbrstr:ident),)+) => {
        $(
            impl<SCKPIN, MISOPIN, MOSIPIN> Spi<$SPI, SCKPIN, MISOPIN, MOSIPIN> {
                /// Creates a new spi instance
                pub fn $spi<F>(
                    spi: $SPI,
                    pins: (SCKPIN, MISOPIN, MOSIPIN),
                    mode: Mode,
                    speed: F,
                    rcc: &mut Rcc,
                ) -> Self
                where
                    SCKPIN: SckPin<$SPI>,
                    MISOPIN: MisoPin<$SPI>,
                    MOSIPIN: MosiPin<$SPI>,
                    F: Into<Hertz>,
                {
                    /* Enable clock for SPI */
                    rcc.regs.$apbenr.modify(|_, w| w.$spiXen().set_bit());

                    /* Reset SPI */
                    rcc.regs.$apbrstr.modify(|_, w| w.$spiXrst().set_bit());
                    rcc.regs.$apbrstr.modify(|_, w| w.$spiXrst().clear_bit());

                    Spi { spi, pins }.spi_init(mode, speed, rcc.clocks)
                }
            }
        )+
    }
}

spi! {
    SPI1: (spi1, spi1en, spi1rst, apb2enr, apb2rstr),
}
#[cfg(any(
    feature = "stm32f030x8",
    feature = "stm32f030xc",
    feature = "stm32f042",
    feature = "stm32f048",
    feature = "stm32f051",
    feature = "stm32f058",
    feature = "stm32f070xb",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
spi! {
    SPI2: (spi2, spi2en, spi2rst, apb1enr, apb1rstr),
}

// It's s needed for the impls, but rustc doesn't recognize that
#[allow(dead_code)]
type SpiRegisterBlock = crate::stm32::spi1::RegisterBlock;

impl<SPI, SCKPIN, MISOPIN, MOSIPIN> Spi<SPI, SCKPIN, MISOPIN, MOSIPIN>
where
    SPI: Deref<Target = SpiRegisterBlock>,
{
    fn spi_init<F>(self: Self, mode: Mode, speed: F, clocks: Clocks) -> Self
    where
        F: Into<Hertz>,
    {
        /* Make sure the SPI unit is disabled so we can configure it */
        self.spi.cr1.modify(|_, w| w.spe().clear_bit());

        // FRXTH: 8-bit threshold on RX FIFO
        // DS: 8-bit data size
        // SSOE: cleared to disable SS output
        //
        // NOTE(unsafe): DS reserved bit patterns are 0b0000, 0b0001, and 0b0010. 0b0111 is valid
        // (reference manual, pp 804)
        self.spi
            .cr2
            .write(|w| unsafe { w.frxth().set_bit().ds().bits(0b0111).ssoe().clear_bit() });

        let br = match clocks.pclk().0 / speed.into().0 {
            0 => unreachable!(),
            1...2 => 0b000,
            3...5 => 0b001,
            6...11 => 0b010,
            12...23 => 0b011,
            24...47 => 0b100,
            48...95 => 0b101,
            96...191 => 0b110,
            _ => 0b111,
        };

        // mstr: master configuration
        // lsbfirst: MSB first
        // ssm: enable software slave management (NSS pin free for other uses)
        // ssi: set nss high = master mode
        // dff: 8 bit frames
        // bidimode: 2-line unidirectional
        // spe: enable the SPI bus
        self.spi.cr1.write(|w| {
            w.cpha()
                .bit(mode.phase == Phase::CaptureOnSecondTransition)
                .cpol()
                .bit(mode.polarity == Polarity::IdleHigh)
                .mstr()
                .set_bit()
                .br()
                .bits(br)
                .lsbfirst()
                .clear_bit()
                .ssm()
                .set_bit()
                .ssi()
                .set_bit()
                .rxonly()
                .clear_bit()
                .bidimode()
                .clear_bit()
                .spe()
                .set_bit()
        });

        self
    }
    pub fn release(self) -> (SPI, (SCKPIN, MISOPIN, MOSIPIN)) {
        (self.spi, self.pins)
    }
}

impl<SPI, SCKPIN, MISOPIN, MOSIPIN> ::embedded_hal::spi::FullDuplex<u8>
    for Spi<SPI, SCKPIN, MISOPIN, MOSIPIN>
where
    SPI: Deref<Target = SpiRegisterBlock>,
{
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Error> {
        let sr = self.spi.sr.read();

        Err(if sr.ovr().bit_is_set() {
            nb::Error::Other(Error::Overrun)
        } else if sr.modf().bit_is_set() {
            nb::Error::Other(Error::ModeFault)
        } else if sr.crcerr().bit_is_set() {
            nb::Error::Other(Error::Crc)
        } else if sr.rxne().bit_is_set() {
            // NOTE(read_volatile) read only 1 byte (the svd2rust API only allows
            // reading a half-word)
            return Ok(unsafe { ptr::read_volatile(&self.spi.dr as *const _ as *const u8) });
        } else {
            nb::Error::WouldBlock
        })
    }

    fn send(&mut self, byte: u8) -> nb::Result<(), Error> {
        let sr = self.spi.sr.read();

        Err(if sr.ovr().bit_is_set() {
            nb::Error::Other(Error::Overrun)
        } else if sr.modf().bit_is_set() {
            nb::Error::Other(Error::ModeFault)
        } else if sr.crcerr().bit_is_set() {
            nb::Error::Other(Error::Crc)
        } else if sr.txe().bit_is_set() {
            // NOTE(write_volatile) see note above
            unsafe { ptr::write_volatile(&self.spi.dr as *const _ as *mut u8, byte) }
            return Ok(());
        } else {
            nb::Error::WouldBlock
        })
    }
}

impl<SPI, SCKPIN, MISOPIN, MOSIPIN> ::embedded_hal::blocking::spi::transfer::Default<u8>
    for Spi<SPI, SCKPIN, MISOPIN, MOSIPIN>
where
    SPI: Deref<Target = SpiRegisterBlock>,
{
}

impl<SPI, SCKPIN, MISOPIN, MOSIPIN> ::embedded_hal::blocking::spi::write::Default<u8>
    for Spi<SPI, SCKPIN, MISOPIN, MOSIPIN>
where
    SPI: Deref<Target = SpiRegisterBlock>,
{
}
