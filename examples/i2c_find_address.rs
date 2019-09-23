#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use crate::hal::{
    prelude::*,
    i2c::I2c,
    stm32,
};

use cortex_m_rt::entry;

/* Example meant for stm32f030xc MCUs with i2c devices connected on PB7 and PB8 */

#[entry]
fn main() -> ! {
    if let Some(p) = stm32::Peripherals::take() {
        cortex_m::interrupt::free(move |cs| {
            let mut flash = p.FLASH;
            let mut rcc = p.RCC.configure().freeze(&mut flash);

            let gpiob = p.GPIOB.split(&mut rcc);

            // Configure pins for I2C
            let sda = gpiob.pb7.into_alternate_af1(cs);
            let scl = gpiob.pb8.into_alternate_af1(cs);

            // Configure I2C with 100kHz rate
            let mut i2c = I2c::i2c1(p.I2C1, (scl, sda), 100.khz(), &mut rcc);

            let mut _devices = 0;
            for add in 0..127 {
                match i2c.write(add, &[]) {
                    Ok(_) => {
                        _devices += 1;
                    },
                    Err(_) => (),
                }
            }

            // Here the variable "_devices" counts how many i2c addresses were a hit
       });
    }

    loop {
        continue;
    }
}
