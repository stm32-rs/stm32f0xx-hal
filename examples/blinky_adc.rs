#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_halt;

use stm32f0xx_hal as hal;

use crate::hal::{adc::Adc, delay::Delay, prelude::*, stm32};

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        cortex_m::interrupt::free(move |cs| {
            let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);

            let gpioa = p.GPIOA.split(&mut rcc);

            // (Re-)configure PA1 as output
            let mut led = gpioa.pa1.into_push_pull_output(cs);

            // (Re-)configure PA0 as analog input
            let mut an_in = gpioa.pa0.into_analog(cs);

            // Get delay provider
            let mut delay = Delay::new(cp.SYST, &rcc);

            // Get access to the ADC
            let mut adc = Adc::new(p.ADC, &mut rcc);

            loop {
                led.toggle();

                let val: u16 = adc.read(&mut an_in).unwrap();

                /* shift the value right by 3, same as divide by 8, reduces
                the 0-4095 range into something approximating 1-512 */
                let time: u16 = (val >> 3) + 1;

                delay.delay_ms(time);
            }
        });
    }

    loop {
        continue;
    }
}
