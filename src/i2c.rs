#[cfg(feature = "stm32f042")]
use crate::stm32::{I2C1, RCC};

use embedded_hal::blocking::i2c::{Write, WriteRead};

#[cfg(feature = "stm32f042")]
use crate::gpio::gpioa::{PA10, PA11, PA12, PA9};
#[cfg(feature = "stm32f042")]
use crate::gpio::gpiob::{PB10, PB11, PB13, PB14, PB6, PB7, PB8, PB9};
#[cfg(feature = "stm32f042")]
use crate::gpio::gpiof::{PF0, PF1};
#[cfg(feature = "stm32f042")]
use crate::gpio::{Alternate, AF1, AF4, AF5};
use crate::time::{KiloHertz, U32Ext};
use core::cmp;

/// I2C abstraction
pub struct I2c<I2C, PINS> {
    i2c: I2C,
    pins: PINS,
}

pub trait Pins<I2c> {}

#[cfg(feature = "stm32f042")]
impl Pins<I2C1> for (PA9<Alternate<AF4>>, PA10<Alternate<AF4>>) {}
#[cfg(feature = "stm32f042")]
impl Pins<I2C1> for (PA11<Alternate<AF5>>, PA12<Alternate<AF5>>) {}
#[cfg(feature = "stm32f042")]
impl Pins<I2C1> for (PB6<Alternate<AF1>>, PB7<Alternate<AF1>>) {}
#[cfg(feature = "stm32f042")]
impl Pins<I2C1> for (PB8<Alternate<AF1>>, PB9<Alternate<AF1>>) {}
#[cfg(feature = "stm32f042")]
impl Pins<I2C1> for (PB10<Alternate<AF1>>, PB11<Alternate<AF1>>) {}
#[cfg(feature = "stm32f042")]
impl Pins<I2C1> for (PB13<Alternate<AF5>>, PB14<Alternate<AF5>>) {}
#[cfg(feature = "stm32f042")]
impl Pins<I2C1> for (PF1<Alternate<AF1>>, PF0<Alternate<AF1>>) {}

#[derive(Debug)]
pub enum Error {
    OVERRUN,
    NACK,
}

#[cfg(feature = "stm32f042")]
impl<PINS> I2c<I2C1, PINS> {
    pub fn i2c1(i2c: I2C1, pins: PINS, speed: KiloHertz) -> Self
    where
        PINS: Pins<I2C1>,
    {
        // NOTE(unsafe) This executes only during initialisation
        let rcc = unsafe { &(*RCC::ptr()) };

        /* Enable clock for I2C1 */
        rcc.apb1enr.modify(|_, w| w.i2c1en().set_bit());

        /* Reset I2C1 */
        rcc.apb1rstr.modify(|_, w| w.i2c1rst().set_bit());
        rcc.apb1rstr.modify(|_, w| w.i2c1rst().clear_bit());

        /* Make sure the I2C unit is disabled so we can configure it */
        i2c.cr1.modify(|_, w| w.pe().clear_bit());

        // Calculate settings for I2C speed modes
        let presc;
        let scldel;
        let sdadel;
        let sclh;
        let scll;

        // We're using HSI here which runs at a fixed 8MHz
        const FREQ: u32 = 8_000_000;

        // Normal I2C speeds use a different scaling than fast mode below
        if speed <= 100_u32.khz() {
            presc = 1;
            scll = cmp::max((((FREQ >> presc) >> 1) / speed.0) - 1, 255) as u8;
            sclh = scll - 4;
            sdadel = 2;
            scldel = 4;
        } else {
            presc = 0;
            scll = cmp::max((((FREQ >> presc) >> 1) / speed.0) - 1, 255) as u8;
            sclh = scll - 6;
            sdadel = 1;
            scldel = 3;
        }

        /* Enable I2C signal generator, and configure I2C for 400KHz full speed */
        i2c.timingr.write(|w| {
            w.presc()
                .bits(presc)
                .scldel()
                .bits(scldel)
                .sdadel()
                .bits(sdadel)
                .sclh()
                .bits(sclh)
                .scll()
                .bits(scll)
        });

        /* Enable the I2C processing */
        i2c.cr1.modify(|_, w| w.pe().set_bit());

        I2c { i2c, pins }
    }

    pub fn release(self) -> (I2C1, PINS) {
        (self.i2c, self.pins)
    }

    fn send_byte(&self, byte: u8) -> Result<(), Error> {
        /* Wait until we're ready for sending */
        while self.i2c.isr.read().txis().bit_is_clear() {}

        /* Push out a byte of data */
        self.i2c.txdr.write(|w| unsafe { w.bits(u32::from(byte)) });

        /* If we received a NACK, then this is an error */
        if self.i2c.isr.read().nackf().bit_is_set() {
            self.i2c
                .icr
                .write(|w| w.stopcf().set_bit().nackcf().set_bit());
            return Err(Error::NACK);
        }

        Ok(())
    }

    fn recv_byte(&self) -> Result<u8, Error> {
        while self.i2c.isr.read().rxne().bit_is_clear() {}
        let value = self.i2c.rxdr.read().bits() as u8;
        Ok(value)
    }
}

#[cfg(feature = "stm32f042")]
impl<PINS> WriteRead for I2c<I2C1, PINS> {
    type Error = Error;

    fn write_read(&mut self, addr: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Error> {
        /* Set up current address, we're trying a "read" command and not going to set anything
         * and make sure we end a non-NACKed read (i.e. if we found a device) properly */
        self.i2c.cr2.modify(|_, w| {
            w.sadd()
                .bits(u16::from(addr) << 1)
                .nbytes()
                .bits(bytes.len() as u8)
                .rd_wrn()
                .clear_bit()
                .autoend()
                .clear_bit()
        });

        /* Send a START condition */
        self.i2c.cr2.modify(|_, w| w.start().set_bit());

        /* Wait until the transmit buffer is empty and there hasn't been either a NACK or STOP
         * being received */
        let mut isr;
        while {
            isr = self.i2c.isr.read();
            isr.txis().bit_is_clear()
                && isr.nackf().bit_is_clear()
                && isr.stopf().bit_is_clear()
                && isr.tc().bit_is_clear()
        } {}

        /* If we received a NACK, then this is an error */
        if isr.nackf().bit_is_set() {
            self.i2c
                .icr
                .write(|w| w.stopcf().set_bit().nackcf().set_bit());
            return Err(Error::NACK);
        }

        for c in bytes {
            self.send_byte(*c)?;
        }

        /* Wait until data was sent */
        while self.i2c.isr.read().tc().bit_is_clear() {}

        /* Set up current address, we're trying a "read" command and not going to set anything
         * and make sure we end a non-NACKed read (i.e. if we found a device) properly */
        self.i2c.cr2.modify(|_, w| {
            w.sadd()
                .bits(u16::from(addr) << 1)
                .nbytes()
                .bits(buffer.len() as u8)
                .rd_wrn()
                .set_bit()
        });

        /* Send a START condition */
        self.i2c.cr2.modify(|_, w| w.start().set_bit());

        /* Send the autoend after setting the start to get a restart */
        self.i2c.cr2.modify(|_, w| w.autoend().set_bit());

        /* Read in all bytes */
        for c in buffer.iter_mut() {
            *c = self.recv_byte()?;
        }

        /* Clear flags if they somehow ended up set */
        self.i2c
            .icr
            .write(|w| w.stopcf().set_bit().nackcf().set_bit());

        Ok(())
    }
}

#[cfg(feature = "stm32f042")]
impl<PINS> Write for I2c<I2C1, PINS> {
    type Error = Error;

    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
        /* Set up current address, we're trying a "read" command and not going to set anything
         * and make sure we end a non-NACKed read (i.e. if we found a device) properly */
        self.i2c.cr2.modify(|_, w| {
            w.sadd()
                .bits(u16::from(addr) << 1)
                .nbytes()
                .bits(bytes.len() as u8)
                .rd_wrn()
                .clear_bit()
                .autoend()
                .set_bit()
        });

        /* Send a START condition */
        self.i2c.cr2.modify(|_, w| w.start().set_bit());

        for c in bytes {
            self.send_byte(*c)?;
        }

        /* Fallthrough is success */
        self.i2c
            .icr
            .write(|w| w.stopcf().set_bit().nackcf().set_bit());
        Ok(())
    }
}
