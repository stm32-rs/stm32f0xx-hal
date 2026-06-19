#![deny(unsafe_code)]
#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _;

use cortex_m_rt::entry;

use stm32f0xx_hal as hal;

use hal::{delay::Delay, pac, prelude::*, pwm};

#[entry]
fn main() -> ! {
    if let Some(mut dp) = pac::Peripherals::take() {
        // Set up the system clock.
        let mut rcc = dp.RCC.configure().sysclk(8.mhz()).freeze(&mut dp.FLASH);

        let gpioa = dp.GPIOA.split(&mut rcc);
        let channels = critical_section::with(move |cs| {
            (
                gpioa.pa8.into_alternate_af2(&cs), // on TIM1_CH1
                gpioa.pa7.into_alternate_af2(&cs), // on TIM1_CH1N
            )
        });

        let pwm = pwm::tim1(dp.TIM1, channels, &mut rcc, 20u32.khz());
        let (mut ch1, mut ch1n) = pwm;
        let max_duty = ch1.get_max_duty();
        ch1.set_duty(max_duty / 2);
        ch1.enable();
        ch1n.enable();

        // simple duty sweep
        if let Some(cp) = cortex_m::Peripherals::take() {
            let mut delay = Delay::new(cp.SYST, &rcc);

            let steps = 100;

            loop {
                for i in 0..steps {
                    ch1.set_duty(max_duty / steps * i);
                    delay.delay_ms(30u16);
                }

                for i in (1..steps).rev() {
                    ch1.set_duty(max_duty / steps * i);
                    delay.delay_ms(30u16);
                }
            }
        }
    }

    // something went wrong when acquiring peripheral access
    loop {
        cortex_m::asm::nop();
    }
}
