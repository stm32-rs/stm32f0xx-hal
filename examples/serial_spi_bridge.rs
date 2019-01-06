#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use crate::hal::prelude::*;
use crate::hal::serial::Serial;
use crate::hal::spi::Spi;
use crate::hal::spi::{Mode, Phase, Polarity};
use crate::hal::stm32;

use nb::block;

use cortex_m_rt::entry;

/// A basic serial to spi example
///
/// If you connect MOSI & MISO pins together, you'll see all characters
/// that you typed in your serial terminal echoed back
///
/// If you connect MISO to GND, you'll see nothing coming back
#[entry]
fn main() -> ! {
    const MODE: Mode = Mode {
        polarity: Polarity::IdleHigh,
        phase: Phase::CaptureOnSecondTransition,
    };

    if let Some(p) = stm32::Peripherals::take() {
        let rcc = p.RCC.constrain();
        let clocks = rcc.cfgr.freeze();
        let gpioa = p.GPIOA.split();

        // Configure pins for SPI
        let sck = gpioa.pa5.into_alternate_af0();
        let miso = gpioa.pa6.into_alternate_af0();
        let mosi = gpioa.pa7.into_alternate_af0();

        // Configure SPI with 1MHz rate
        let mut spi = Spi::spi1(p.SPI1, (sck, miso, mosi), MODE, 1.mhz(), clocks);

        let tx = gpioa.pa9.into_alternate_af1();
        let rx = gpioa.pa10.into_alternate_af1();

        let serial = Serial::usart1(p.USART1, (tx, rx), 115_200.bps(), clocks);

        let (mut tx, mut rx) = serial.split();

        loop {
            let serial_received = block!(rx.read()).unwrap();

            block!(spi.send(serial_received)).ok();

            let spi_received = block!(spi.read()).unwrap();

            block!(tx.write(spi_received)).ok();
        }
    }

    loop {
        continue;
    }
}
