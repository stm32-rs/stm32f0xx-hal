#![deny(unused_imports)]
#![no_main]
#![no_std]

use cortex_m_rt as rt;
use panic_halt as _;

use stm32f0xx_hal as hal;

use crate::hal::dac::*;
use crate::hal::pac;
use crate::hal::prelude::*;

use rt::entry;

enum Direction {
    Upcounting,
    Downcounting,
}

#[entry]
fn main() -> ! {
    if let (Some(mut dp), Some(_cp)) = (pac::Peripherals::take(), cortex_m::Peripherals::take()) {
        let mut rcc = dp.RCC.configure().sysclk(8.mhz()).freeze(&mut dp.FLASH);
        let gpioa = dp.GPIOA.split(&mut rcc);

        let pa4 = cortex_m::interrupt::free(move |_| {
            // SAFETY: We are in a critical section, but the `cortex_m` critical section
            // token is not compatible with the `bare_metal` token. Future version of the
            // `cortex_m` crate will not supply *any* token to this callback!
            let cs = unsafe { &bare_metal::CriticalSection::new() };
            gpioa.pa4.into_analog(cs)
        });

        let mut dac = dac(dp.DAC, pa4, &mut rcc);

        dac.enable();

        let mut dir = Direction::Upcounting;
        let mut val = 0;

        dac.set_value(2058);
        cortex_m::asm::bkpt();

        dac.set_value(4095);
        cortex_m::asm::bkpt();

        loop {
            dac.set_value(val);
            match val {
                0 => dir = Direction::Upcounting,
                4095 => dir = Direction::Downcounting,
                _ => (),
            };

            match dir {
                Direction::Upcounting => val += 1,
                Direction::Downcounting => val -= 1,
            }
        }
    }

    loop {
        continue;
    }
}
