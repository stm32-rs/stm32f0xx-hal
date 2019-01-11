#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use core::fmt::Write;
use stm32f0xx_hal as hal;

use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::hal::serial::Serial;
use crate::hal::stm32;
use crate::hal::time::Hertz;
use crate::hal::watchdog::Watchdog;

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        cortex_m::interrupt::free(move |cs| {
            let mut flash = p.FLASH;
            let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut flash);

            let gpioa = p.GPIOA.split(&mut rcc);
            let dbgmcu = p.DBGMCU;

            // Disable the watchdog when the cpu is stopped under debug
            dbgmcu.apb1_fz.modify(|_, w| w.dbg_iwdg_stop().set_bit());

            let mut watchdog = Watchdog::new(p.IWDG);

            // Get delay provider
            let mut delay = Delay::new(cp.SYST, &rcc);

            let tx = gpioa.pa9.into_alternate_af1(cs);

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
        });
    }

    loop {
        continue;
    }
}
