use core::ptr;

use nb;

pub use embedded_hal::spi::{Mode, Phase, Polarity};

#[cfg(any(feature = "stm32f042", feature = "stm32f030"))]
use crate::stm32::{RCC, SPI1};

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
pub struct Spi<SPI, PINS> {
    spi: SPI,
    pins: PINS,
}

pub trait Pins<Spi> {}

#[cfg(any(feature = "stm32f042", feature = "stm32f030"))]
impl Pins<SPI1>
    for (
        gpioa::PA5<Alternate<AF0>>,
        gpioa::PA6<Alternate<AF0>>,
        gpioa::PA7<Alternate<AF0>>,
    )
{}
#[cfg(any(feature = "stm32f042", feature = "stm32f030"))]
impl Pins<SPI1>
    for (
        gpiob::PB3<Alternate<AF0>>,
        gpiob::PB4<Alternate<AF0>>,
        gpiob::PB5<Alternate<AF0>>,
    )
{}

#[cfg(feature = "stm32f030x6")]
impl Pins<SPI1>
    for (
        gpiob::PB13<Alternate<AF0>>,
        gpiob::PB14<Alternate<AF0>>,
        gpiob::PB15<Alternate<AF0>>,
    )
{
}

#[cfg(any(feature = "stm32f042", feature = "stm32f030"))]
impl<PINS> Spi<SPI1, PINS> {
    pub fn spi1<F>(spi: SPI1, pins: PINS, mode: Mode, speed: F, clocks: Clocks) -> Self
    where
        PINS: Pins<SPI1>,
        F: Into<Hertz>,
    {
        // NOTE(unsafe) This executes only during initialisation
        let rcc = unsafe { &(*RCC::ptr()) };

        /* Enable clock for SPI1 */
        rcc.apb2enr.modify(|_, w| w.spi1en().set_bit());

        /* Reset SPI1 */
        rcc.apb2rstr.modify(|_, w| w.spi1rst().set_bit());
        rcc.apb2rstr.modify(|_, w| w.spi1rst().clear_bit());

        /* Make sure the SPI unit is disabled so we can configure it */
        spi.cr1.modify(|_, w| w.spe().clear_bit());

        // FRXTH: 8-bit threshold on RX FIFO
        // DS: 8-bit data size
        // SSOE: cleared to disable SS output
        //
        // NOTE(unsafe): DS reserved bit patterns are 0b0000, 0b0001, and 0b0010. 0b0111 is valid
        // (reference manual, pp 804)
        spi.cr2
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
        spi.cr1.write(|w| unsafe {
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

        Spi { spi, pins }
    }

    pub fn release(self) -> (SPI1, PINS) {
        (self.spi, self.pins)
    }
}

#[cfg(any(feature = "stm32f042", feature = "stm32f030"))]
impl<PINS> ::embedded_hal::spi::FullDuplex<u8> for Spi<SPI1, PINS> {
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

#[cfg(any(feature = "stm32f042", feature = "stm32f030"))]
impl<PINS> ::embedded_hal::blocking::spi::transfer::Default<u8> for Spi<SPI1, PINS> {}
#[cfg(any(feature = "stm32f042", feature = "stm32f030"))]
impl<PINS> ::embedded_hal::blocking::spi::write::Default<u8> for Spi<SPI1, PINS> {}
