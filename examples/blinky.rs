#![no_main]
#![no_std]

extern crate cortex_m_rt;
extern crate panic_halt;

extern crate stm32f0xx_hal as hal;

use hal::prelude::*;
use hal::stm32;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let Some(p) = stm32::Peripherals::take() {
        let gpioa = p.GPIOA.split();

        /* (Re-)configure PA1 as output */
        let mut led = gpioa.pa1.into_push_pull_output();

        loop {
            /* Turn PA1 on a million times in a row */
            for _ in 0..1_000_000 {
                led.set_high();
            }
            /* Then turn PA1 off a million times in a row */
            for _ in 0..1_000_000 {
                led.set_low();
            }
        }
    }

    loop {
        continue;
    }
}
