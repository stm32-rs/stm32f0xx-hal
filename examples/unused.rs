//! This is not intended to be used on a real system
#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use crate::hal::i2c::*;
use crate::hal::prelude::*;
use crate::hal::spi::*;
use crate::hal::stm32;

use cortex_m_rt::entry;
#[entry]
fn main() -> ! {
    const MODE: Mode = Mode {
        polarity: Polarity::IdleHigh,
        phase: Phase::CaptureOnSecondTransition,
    };

    if let Some(p) = stm32::Peripherals::take() {
        let mut flash = p.FLASH;
        let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut flash);

        let pins = unsafe { (NoSck::new(), NoMiso::new(), NoMosi::new()) };
        let _ = Spi::spi1(p.SPI1, pins, MODE, 100_000.hz(), &mut rcc);

        let pins = unsafe { (NoScl::new(), NoSda::new()) };
        let _ = I2c::i2c1(p.I2C1, pins, 400.khz(), &mut rcc);
    }
    loop {
        continue;
    }
}
