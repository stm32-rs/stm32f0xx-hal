#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal as hal;

use crate::hal::{adc::Adc, delay::Delay, pac, prelude::*};

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (pac::Peripherals::take(), Peripherals::take()) {
        let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);

        let gpioa = p.GPIOA.split(&mut rcc);

        let (mut led, mut an_in) = cortex_m::interrupt::free(move |_| {
            // SAFETY: We are in a critical section, but the `cortex_m` critical section
            // token is not compatible with the `bare_metal` token. Future version of the
            // `cortex_m` crate will not supply *any* token to this callback!
            let cs = unsafe { &bare_metal::CriticalSection::new() };

            (
                // (Re-)configure PA1 as output
                gpioa.pa1.into_push_pull_output(cs),
                // (Re-)configure PA0 as analog input
                gpioa.pa0.into_analog(cs),
            )
        });

        // Get delay provider
        let mut delay = Delay::new(cp.SYST, &rcc);

        // Get access to the ADC
        let mut adc = Adc::new(p.ADC, &mut rcc);

        loop {
            led.toggle().ok();

            let time: u16 = if let Ok(val) = adc.read(&mut an_in) as Result<u16, _> {
                /* shift the value right by 3, same as divide by 8, reduces
                the 0-4095 range into something approximating 1-512 */
                (val >> 3) + 1
            } else {
                1000
            };

            delay.delay_ms(time);
        }
    }

    loop {
        continue;
    }
}
