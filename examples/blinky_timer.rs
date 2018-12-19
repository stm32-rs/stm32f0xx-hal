#![no_main]
#![no_std]

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
        let gpioa = p.GPIOA.split();
        /* (Re-)configure PA1 as output */
        let mut led = gpioa.pa1.into_push_pull_output();

        /* Constrain clocking registers */
        let rcc = p.RCC.constrain();

        /* Configure clock to 8 MHz (i.e. the default) and freeze it */
        let clocks = rcc.cfgr.sysclk(8.mhz()).freeze();

        let mut timer = Timer::tim1(p.TIM1, Hertz(1), clocks);

        loop {
            led.toggle();
            block!(timer.wait()).ok();
        }
    }

    loop {
        continue;
    }
}
