#![deny(unused_imports)]
#![no_main]
#![no_std]

use cortex_m;
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
        cortex_m::interrupt::free(move |cs| {
            let mut rcc = dp.RCC.configure().sysclk(8.mhz()).freeze(&mut dp.FLASH);

            let gpioa = dp.GPIOA.split(&mut rcc);
            let mut dac = dac(dp.DAC, gpioa.pa4.into_analog(cs), &mut rcc);

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
        });
    }

    loop {
        continue;
    }
}
