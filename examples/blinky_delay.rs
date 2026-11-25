#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal as hal;

use crate::hal::{delay::Delay, pac, prelude::*};

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (pac::Peripherals::take(), Peripherals::take()) {
        let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);

        let gpioa = p.GPIOA.split(&mut rcc);

        // (Re-)configure PA1 as output
        let mut led = critical_section::with(move |cs| gpioa.pa1.into_push_pull_output(&cs));

        // Get delay provider
        let mut delay = Delay::new(cp.SYST, &rcc);

        loop {
            led.toggle().ok();
            delay.delay_ms(1_000_u16);
        }
    }

    loop {
        continue;
    }
}
