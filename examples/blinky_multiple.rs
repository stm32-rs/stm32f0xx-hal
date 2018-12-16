#![no_main]
#![no_std]

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
        let gpioa = p.GPIOA.split();
        let gpiob = p.GPIOB.split();

        /* (Re-)configure PA1 as output */
        let mut led1 = gpioa.pa1.into_push_pull_output();

        /* (Re-)configure PB1 as output */
        let mut led2 = gpiob.pb1.into_push_pull_output();

        /* Constrain clocking registers */
        let rcc = p.RCC.constrain();

        /* Configure clock to 8 MHz (i.e. the default) and freeze it */
        let clocks = rcc.cfgr.sysclk(8.mhz()).freeze();

        /* Get delay provider */
        let mut delay = Delay::new(cp.SYST, clocks);

        /* Store them together */
        let mut leds = [led1.downgrade().downgrade(), led2.downgrade().downgrade()];
        loop {
            leds[0].set_high();
            leds[1].set_high();
            delay.delay_ms(1_000_u16);

            leds[0].set_low();
            leds[1].set_low();
            delay.delay_ms(1_000_u16);
        }
    }

    loop {
        continue;
    }
}
