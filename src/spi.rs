use core::ops::Deref;
use core::ptr;

use nb;

pub use embedded_hal::spi::{Mode, Phase, Polarity};

use crate::stm32;
// TODO Put this inside the macro
// Currently that causes a compiler panic
#[cfg(any(feature = "stm32f042", feature = "stm32f030", feature = "stm32f070"))]
use crate::stm32::SPI1;
#[cfg(any(
    feature = "stm32f030x8",
    feature = "stm32f030xc",
    feature = "stm32f070xb"
))]
use crate::stm32::SPI2;

use crate::gpio::*;
use crate::rcc::Clocks;
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
                impl SckPin<stm32::$SPI> for $sck {}
            )+
            $(
                impl MisoPin<stm32::$SPI> for $miso {}
            )+
            $(
                impl MosiPin<stm32::$SPI> for $mosi {}
            )+
        )+
    }
}

#[cfg(any(feature = "stm32f042", feature = "stm32f030", feature = "stm32f070"))]
spi_pins! {
    SPI1 => {
        sck => [gpioa::PA5<Alternate<AF0>>, gpiob::PB3<Alternate<AF0>>],
        miso => [gpioa::PA6<Alternate<AF0>>, gpiob::PB4<Alternate<AF0>>],
        mosi => [gpioa::PA7<Alternate<AF0>>, gpiob::PB5<Alternate<AF0>>],
    }
}
#[cfg(feature = "stm32f030x6")]
spi_pins! {
    SPI1 => {
        sck => [gpiob::PB13<Alternate<AF0>>],
        miso => [gpiob::PB14<Alternate<AF0>>],
        mosi => [gpiob::PB15<Alternate<AF0>>],
    }
}
#[cfg(any(
    feature = "stm32f030x8",
    feature = "stm32f030xc",
    feature = "stm32f070xb"
))]
spi_pins! {
    SPI2 => {
        sck => [gpiob::PB13<Alternate<AF0>>],
        miso => [gpiob::PB14<Alternate<AF0>>],
        mosi => [gpiob::PB15<Alternate<AF0>>],
    }
}
#[cfg(any(feature = "stm32f030xc", feature = "stm32f070xb"))]
spi_pins! {
    SPI2 => {
        sck => [gpiob::PB10<Alternate<AF5>>],
        miso => [gpioc::PC2<Alternate<AF1>>],
        mosi => [gpioc::PC3<Alternate<AF1>>],
    }
}

macro_rules! spi {
    ($($SPI:ident: ($spi:ident, $spiXen:ident, $spiXrst:ident, $apbenr:ident, $apbrstr:ident),)+) => {
        $(
            impl<SCKPIN, MISOPIN, MOSIPIN> Spi<$SPI, SCKPIN, MISOPIN, MOSIPIN> {
                pub fn $spi<F>(
                    spi: $SPI,
                    pins: (SCKPIN, MISOPIN, MOSIPIN),
                    mode: Mode,
                    speed: F,
                    clocks: Clocks,
                ) -> Self
                where
                    SCKPIN: SckPin<$SPI>,
                    MISOPIN: MisoPin<$SPI>,
                    MOSIPIN: MosiPin<$SPI>,
                    F: Into<Hertz>,
                {
                    // NOTE(unsafe) This executes only during initialisation
                    let rcc = unsafe { &(*stm32::RCC::ptr()) };

                    /* Enable clock for SPI */
                    rcc.$apbenr.modify(|_, w| w.$spiXen().set_bit());

                    /* Reset SPI */
                    rcc.$apbrstr.modify(|_, w| w.$spiXrst().set_bit());
                    rcc.$apbrstr.modify(|_, w| w.$spiXrst().clear_bit());
                    Spi { spi, pins }.spi_init(mode, speed, clocks)
                }
            }
        )+
    }
}

#[cfg(any(feature = "stm32f042", feature = "stm32f030", feature = "stm32f070"))]
spi! {
    SPI1: (spi1, spi1en, spi1rst, apb2enr, apb2rstr),
}
#[cfg(any(
    feature = "stm32f030x8",
    feature = "stm32f030xc",
    feature = "stm32f070xb"
))]
spi! {
    SPI2: (spi2, spi2en, spi2rst, apb1enr, apb1rstr),
}

// It's s needed for the impls, but rustc doesn't recognize that
#[allow(dead_code)]
type SpiRegisterBlock = stm32::spi1::RegisterBlock;

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
        self.spi.cr1.write(|w| unsafe {
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
