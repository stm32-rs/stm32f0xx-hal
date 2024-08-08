#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal as hal;

use crate::hal::{
    pac,
    prelude::*,
    spi::Spi,
    spi::{Mode, Phase, Polarity},
};

use cortex_m_rt::entry;

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

        // Configure pins for SPI
        let (sck, miso, mosi) = cortex_m::interrupt::free(move |_| {
            // SAFETY: We are in a critical section, but the `cortex_m` critical section
            // token is not compatible with the `bare_metal` token. Future version of the
            // `cortex_m` crate will not supply *any* token to this callback!
            let cs = unsafe { &bare_metal::CriticalSection::new() };
            (
                gpioa.pa5.into_alternate_af0(cs),
                gpioa.pa6.into_alternate_af0(cs),
                gpioa.pa7.into_alternate_af0(cs),
            )
        });

        // Configure SPI with 100kHz rate
        let mut spi = Spi::spi1(p.SPI1, (sck, miso, mosi), MODE, 100_000.hz(), &mut rcc);

        // Cycle through colors on 16 chained APA102C LEDs
        loop {
            for r in 0..255 {
                let _ = spi.write(&[0, 0, 0, 0]);
                for _i in 0..16 {
                    let _ = spi.write(&[0b1110_0001, 0, 0, r]);
                }
                let _ = spi.write(&[0xFF, 0xFF, 0xFF, 0xFF]);
            }
            for b in 0..255 {
                let _ = spi.write(&[0, 0, 0, 0]);
                for _i in 0..16 {
                    let _ = spi.write(&[0b1110_0001, b, 0, 0]);
                }
                let _ = spi.write(&[0xFF, 0xFF, 0xFF, 0xFF]);
            }
            for g in 0..255 {
                let _ = spi.write(&[0, 0, 0, 0]);
                for _i in 0..16 {
                    let _ = spi.write(&[0b1110_0001, 0, g, 0]);
                }
                let _ = spi.write(&[0xFF, 0xFF, 0xFF, 0xFF]);
            }
        }
    }

    loop {
        continue;
    }
}
