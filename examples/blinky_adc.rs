#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_halt;

use stm32f0xx_hal as hal;

use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::hal::stm32;

use crate::hal::adc::Adc;

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        let gpioa = p.GPIOA.split();

        /* (Re-)configure PA1 as output */
        let mut led = gpioa.pa1.into_push_pull_output();

        /* (Re-)configure PA0 as analog in */
        let mut an_in = gpioa.pa0.into_analog();

        /* Constrain clocking registers */
        let rcc = p.RCC.constrain();

        /* Configure clock to 8 MHz (i.e. the default) and freeze it */
        let clocks = rcc.cfgr.sysclk(8.mhz()).freeze();

        /* Get delay provider */
        let mut delay = Delay::new(cp.SYST, clocks);

        let mut adc = Adc::new(p.ADC);

        loop {
            led.toggle();

            let val: u16 = adc.read(&mut an_in).unwrap();

            /* shift the value right by 3, same as divide by 8, reduces
            the 0-4095 range into something approximating 1-512 */
            let time: u16 = (val >> 3) + 1;

            delay.delay_ms(time);
        }
    }

    loop {
        continue;
    }
}
