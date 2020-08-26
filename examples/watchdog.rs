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
        let tx = cortex_m::interrupt::free(move |cs| gpioa.pa9.into_alternate_af1(cs));

        // Obtain a serial peripheral with for unidirectional communication
        let mut serial = Serial::usart1tx(p.USART1, tx, 115_200.bps(), &mut rcc);

        serial.write_str("RESET \r\n").ok();

        watchdog.try_start(Hertz(1)).ok();
        delay.try_delay_ms(500_u16).ok();
        watchdog.try_feed().ok();
        delay.try_delay_ms(500_u16).ok();
        watchdog.try_feed().ok();
        delay.try_delay_ms(500_u16).ok();
        serial.write_str("This will get printed \r\n").ok();
        watchdog.try_feed().ok();

        // Now a reset happens while delaying
        delay.try_delay_ms(1500_u16).ok();
        serial.write_str("This won't\r\n").ok();
    }

    loop {
        continue;
    }
}
