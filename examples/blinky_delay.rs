#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::hal::stm32;

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        cortex_m::interrupt::free(move |cs| {
            let mut flash = p.FLASH;
            let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut flash);

            let gpioa = p.GPIOA.split(&mut rcc);

            /* (Re-)configure PA1 as output */
            let mut led = gpioa.pa1.into_push_pull_output(cs);

            /* Get delay provider */
            let mut delay = Delay::new(cp.SYST, &rcc);

            loop {
                led.toggle();
                delay.delay_ms(1_000_u16);
            }
        });
    }

    loop {
        continue;
    }
}
