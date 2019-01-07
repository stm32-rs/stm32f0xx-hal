#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use crate::hal::prelude::*;
use crate::hal::stm32;
use crate::hal::time::*;
use crate::hal::timers::*;

use cortex_m_rt::entry;
use nb::block;

#[entry]
fn main() -> ! {
    if let Some(p) = stm32::Peripherals::take() {
        cortex_m::interrupt::free(move |cs| {
            let mut flash = p.FLASH;
            let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut flash);

            let gpioa = p.GPIOA.split(&mut rcc);
            /* (Re-)configure PA1 as output */
            let mut led = gpioa.pa1.into_push_pull_output(cs);

            let mut timer = Timer::tim1(p.TIM1, Hertz(1), &mut rcc);

            loop {
                led.toggle();
                block!(timer.wait()).ok();
            }
        });
    }

    loop {
        continue;
    }
}
