#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use crate::hal::prelude::*;
use crate::hal::stm32;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let Some(p) = stm32::Peripherals::take() {
        cortex_m::interrupt::free(move |cs| {
            let mut flash = p.FLASH;
            let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut flash);

            let gpioa = p.GPIOA.split(&mut rcc);

            /* (Re-)configure PA1 as output */
            let mut led = gpioa.pa1.into_push_pull_output(cs);

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
        });
    }

    loop {
        continue;
    }
}
