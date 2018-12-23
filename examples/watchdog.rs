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
        let gpioa = p.GPIOA.split();
        let rcc = p.RCC.constrain();
        let dbgmcu = p.DBGMCU;

        // Disable the watchdog when the cpu is stopped under debug
        dbgmcu.apb1_fz.modify(|_, w| w.dbg_iwdg_stop().set_bit());

        let mut watchdog = Watchdog::new(p.IWDG);
        let clocks = rcc.cfgr.sysclk(8.mhz()).freeze();

        // Get delay provider
        let mut delay = Delay::new(cp.SYST, clocks);

        let tx = gpioa.pa9.into_alternate_af1();
        let rx = gpioa.pa10.into_alternate_af1();

        let serial = Serial::usart1(p.USART1, (tx, rx), 115_200.bps(), clocks);

        let (mut tx, _rx) = serial.split();
        tx.write_str("RESET \r\n").ok();

        watchdog.start(Hertz(1));
        delay.delay_ms(500_u16);
        watchdog.feed();
        delay.delay_ms(500_u16);
        watchdog.feed();
        delay.delay_ms(500_u16);
        tx.write_str("This will get printed \r\n").ok();
        watchdog.feed();

        // Now a reset happens while delaying
        delay.delay_ms(1500_u16);
        tx.write_str("This won't\r\n").ok();
    }

    loop {
        continue;
    }
}
