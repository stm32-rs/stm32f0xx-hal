//! API for the IWDG
//!
//! You can activate the watchdog by calling `start` or the setting appropriate
//! device option bit when programming.
//!
//! After activating the watchdog, you'll have to regularly `feed` the watchdog.
//! If more time than `timeout` has gone by since the last `feed`, your
//! microcontroller will be reset.
//!
//! This is useful if you fear that your program may get stuck. In that case it
//! won't feed the watchdog anymore, the watchdog will reset the microcontroller
//! and thus your program will function again.
//!
//! **Attention**:
//!
//! The IWDG runs on a separate 40kHz low-accuracy clock (30kHz-60kHz). You may
//! want to some buffer in your interval.
//!
//! Per default the iwdg continues to run even when you stopped execution of code via a debugger.
//! You may want to disable the watchdog when the cpu is stopped
//!
//! ``` ignore
//! let dbgmcu = p.DBGMCU;
//! dbgmcu.apb1_fz.modify(|_, w| w.dbg_iwdg_stop().set_bit());
//! ```
//!
//! # Example
//! ``` no_run
//! use stm32f0xx_hal as hal;
//!
//! use crate::hal::stm32;
//! use crate::hal::prelude::*;
//! use crate::hal:watchdog::Watchdog;
//! use crate::hal:time::Hertz;
//!
//! let mut p = stm32::Peripherals::take().unwrap();
//!
//! let mut iwdg = Watchdog::new(p.iwdg);
//! iwdg.start(Hertz(100));
//! loop {}
//! // Whoops, got stuck, the watchdog issues a reset after 10 ms
//! iwdg.feed();
//! ```
use embedded_hal::watchdog;

use crate::stm32::IWDG;
use crate::time::Hertz;

/// Watchdog instance
pub struct Watchdog {
    iwdg: IWDG,
}

impl watchdog::Watchdog for Watchdog {
    /// Feed the watchdog, so that at least one `period` goes by before the next
    /// reset
    fn feed(&mut self) {
        self.iwdg.kr.write(|w| w.key().reset());
    }
}

/// Timeout configuration for the IWDG
#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct IwdgTimeout {
    psc: u8,
    reload: u16,
}

impl Into<IwdgTimeout> for Hertz {
    /// This converts the value so it's usable by the IWDG
    /// Due to conversion losses, the specified frequency is a maximum
    ///
    /// It can also only represent values < 10000 Hertz
    fn into(self) -> IwdgTimeout {
        let mut time = 40_000 / 4 / self.0;
        let mut psc = 0;
        let mut reload = 0;
        while psc < 7 {
            reload = time;
            if reload < 0x1000 {
                break;
            }
            psc += 1;
            time /= 2;
        }
        // As we get an integer value, reload is always below 0xFFF
        let reload = reload as u16;
        IwdgTimeout { psc, reload }
    }
}

impl Watchdog {
    pub fn new(iwdg: IWDG) -> Self {
        Self { iwdg }
    }
}

impl watchdog::WatchdogEnable for Watchdog {
    type Time = IwdgTimeout;
    fn start<T>(&mut self, period: T)
    where
        T: Into<IwdgTimeout>,
    {
        let time: IwdgTimeout = period.into();
        // Feed the watchdog in case it's already running
        // (Waiting for the registers to update takes sometime)
        self.iwdg.kr.write(|w| w.key().reset());
        // Enable the watchdog
        self.iwdg.kr.write(|w| w.key().start());
        self.iwdg.kr.write(|w| w.key().enable());
        // Wait until it's safe to write to the registers
        while self.iwdg.sr.read().pvu().bit() {}
        self.iwdg.pr.write(|w| w.pr().bits(time.psc));
        while self.iwdg.sr.read().rvu().bit() {}
        self.iwdg.rlr.write(|w| w.rl().bits(time.reload));
        // Wait until the registers are updated before issuing a reset with
        // (potentially false) values
        while self.iwdg.sr.read().bits() != 0 {}
        self.iwdg.kr.write(|w| w.key().reset());
    }
}
