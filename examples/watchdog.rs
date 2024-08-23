#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal as hal;

use crate::hal::{delay::Delay, pac, prelude::*, serial::Serial, time::Hertz, watchdog::Watchdog};

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

use core::fmt::Write;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (pac::Peripherals::take(), Peripherals::take()) {
        let mut flash = p.FLASH;
        let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut flash);

        let gpioa = p.GPIOA.split(&mut rcc);
        let dbgmcu = p.DBGMCU;

        // Disable the watchdog when the cpu is stopped under debug
        dbgmcu.apb1_fz.modify(|_, w| w.dbg_iwdg_stop().set_bit());

        let mut watchdog = Watchdog::new(p.IWDG);

        // Get delay provider
        let mut delay = Delay::new(cp.SYST, &rcc);

        // Configure serial TX pin
        let tx = cortex_m::interrupt::free(move |_| {
            // SAFETY: We are in a critical section, but the `cortex_m` critical section
            // token is not compatible with the `bare_metal` token. Future version of the
            // `cortex_m` crate will not supply *any* token to this callback!
            let cs = unsafe { &bare_metal::CriticalSection::new() };
            gpioa.pa9.into_alternate_af1(cs)
        });

        // Obtain a serial peripheral with for unidirectional communication
        let mut serial = Serial::usart1tx(p.USART1, tx, 115_200.bps(), &mut rcc);

        serial.write_str("RESET \r\n").ok();

        watchdog.start(Hertz(1));
        delay.delay_ms(500_u16);
        watchdog.feed();
        delay.delay_ms(500_u16);
        watchdog.feed();
        delay.delay_ms(500_u16);
        serial.write_str("This will get printed \r\n").ok();
        watchdog.feed();

        // Now a reset happens while delaying
        delay.delay_ms(1500_u16);
        serial.write_str("This won't\r\n").ok();
    }

    loop {
        continue;
    }
}
