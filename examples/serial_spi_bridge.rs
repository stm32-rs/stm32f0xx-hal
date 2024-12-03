#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal as hal;

use crate::hal::{
    pac,
    prelude::*,
    serial::Serial,
    spi::Spi,
    spi::{Mode, Phase, Polarity},
};

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

    if let Some(p) = pac::Peripherals::take() {
        let mut flash = p.FLASH;
        let mut rcc = p.RCC.configure().freeze(&mut flash);

        let gpioa = p.GPIOA.split(&mut rcc);

        let (sck, miso, mosi, tx, rx) = critical_section::with(move |cs| {
            (
                // SPI pins
                gpioa.pa5.into_alternate_af0(&cs),
                gpioa.pa6.into_alternate_af0(&cs),
                gpioa.pa7.into_alternate_af0(&cs),
                // USART pins
                gpioa.pa9.into_alternate_af1(&cs),
                gpioa.pa10.into_alternate_af1(&cs),
            )
        });

        // Configure SPI with 1MHz rate
        let mut spi = Spi::spi1(p.SPI1, (sck, miso, mosi), MODE, 1.mhz(), &mut rcc);

        let serial = Serial::usart1(p.USART1, (tx, rx), 115_200.bps(), &mut rcc);

        let (mut tx, mut rx) = serial.split();

        let mut data = [0];
        loop {
            let serial_received = block!(rx.read()).unwrap();
            spi.write(&[serial_received]).ok();
            let spi_received = spi.transfer(&mut data).unwrap();
            block!(tx.write(spi_received[0])).ok();
        }
    }

    loop {
        continue;
    }
}
