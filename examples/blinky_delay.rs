#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate panic_halt;

extern crate stm32f0xx_hal as hal;

use hal::delay::Delay;
use hal::prelude::*;
use hal::stm32;

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        let gpioa = p.GPIOA.split();

        /* (Re-)configure PA1 as output */
        let mut led = gpioa.pa1.into_push_pull_output();

        /* Constrain clocking registers */
        let mut rcc = p.RCC.constrain();

        /* Configure clock to 8 MHz (i.e. the default) and freeze it */
        let clocks = rcc.cfgr.sysclk(8.mhz()).freeze();

        /* Get delay provider */
        let mut delay = Delay::new(cp.SYST, clocks);

        loop {
            led.set_high();
            delay.delay_ms(1_000_u16);

            led.set_low();
            delay.delay_ms(1_000_u16);
        }
    }

    loop {
        continue;
    }
}
