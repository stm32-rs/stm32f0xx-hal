#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use crate::hal::prelude::*;
use crate::hal::serial::Serial;
use crate::hal::stm32;

use nb::block;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let Some(p) = stm32::Peripherals::take() {
        cortex_m::interrupt::free(move |cs| {
            let mut flash = p.FLASH;
            let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut flash);

            let gpioa = p.GPIOA.split(&mut rcc);

            let tx = gpioa.pa9.into_alternate_af1(cs);
            let rx = gpioa.pa10.into_alternate_af1(cs);

            let mut serial = Serial::usart1(p.USART1, (tx, rx), 115_200.bps(), &mut rcc);

            loop {
                let received = block!(serial.read()).unwrap();
                block!(serial.write(received)).ok();
            }
        });
    }

    loop {
        continue;
    }
}
