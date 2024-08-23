#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal as hal;

use crate::hal::{pac, prelude::*, time::Hertz, timers::*};

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let Some(mut p) = pac::Peripherals::take() {
        let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);

        let gpioa = p.GPIOA.split(&mut rcc);

        // (Re-)configure PA1 as output
        let mut led = cortex_m::interrupt::free(move |_| {
            // SAFETY: We are in a critical section, but the `cortex_m` critical section
            // token is not compatible with the `bare_metal` token. Future version of the
            // `cortex_m` crate will not supply *any* token to this callback!
            let cs = unsafe { &bare_metal::CriticalSection::new() };
            gpioa.pa1.into_push_pull_output(cs)
        });

        // Set up a timer expiring after 1s
        let mut timer = Timer::tim1(p.TIM1, Hertz(1), &mut rcc);

        loop {
            led.toggle().ok();

            // Wait for the timer to expire
            nb::block!(timer.wait()).ok();
        }
    }

    loop {
        continue;
    }
}
