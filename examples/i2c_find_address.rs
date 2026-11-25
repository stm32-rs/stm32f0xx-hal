#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal as hal;

use crate::hal::{i2c::I2c, pac, prelude::*};

use cortex_m_rt::entry;

/* Example meant for stm32f030xc MCUs with i2c devices connected on PB7 and PB8 */

#[entry]
fn main() -> ! {
    if let Some(p) = pac::Peripherals::take() {
        critical_section::with(move |cs| {
            let mut flash = p.FLASH;
            let mut rcc = p.RCC.configure().freeze(&mut flash);

            let gpiob = p.GPIOB.split(&mut rcc);

            // Configure pins for I2C
            let sda = gpiob.pb7.into_alternate_af1(&cs);
            let scl = gpiob.pb8.into_alternate_af1(&cs);

            // Configure I2C with 100kHz rate
            let mut i2c = I2c::i2c1(p.I2C1, (scl, sda), 100.khz(), &mut rcc);

            let mut _devices = 0;
            // I2C addresses are 7-bit wide, covering the 0-127 range
            for add in 0..=127 {
                // The write method sends the specified address and checks for acknowledgement;
                // if no ack is given by the slave device the result is Err(), otherwise Ok()
                // Since we only care for an acknowledgement the data sent can be empty
                if i2c.write(add, &[]).is_ok() {
                    _devices += 1;
                }
            }

            // Here the variable "_devices" counts how many i2c addresses were a hit
        });
    }

    loop {
        continue;
    }
}
