//! API for delays with the systick timer
//!
//! Please be aware of potential overflows.
//! For example, the maximum delay with 48MHz is around 89 seconds
//!
//! Consider using the timers api as a more flexible interface
//!
//! # Example
//!
//! ``` no_run
//! use stm32f0xx_hal as hal;
//!
//! use crate::hal::stm32;
//! use crate::hal::prelude::*;
//! use crate::hal::delay::Delay;
//! use cortex_m::peripheral::Peripherals;
//!
//! let mut p = stm32::Peripherals::take().unwrap();
//! let mut cp = cortex_m::Peripherals::take().unwrap();
//!
//! let clocks = p.RCC.constrain().cfgr.freeze();
//! let mut delay = Delay::new(cp.SYST, clocks);
//! loop {
//!     delay.delay_ms(1_000_u16);
//! }
//! ```

use cast::{u16, u32};
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;

use crate::rcc::Clocks;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};

/// System timer (SysTick) as a delay provider
#[derive(Clone)]
pub struct Delay {
    scale: Scale,
}

#[derive(Clone)]
enum Scale {
    Mult(u32),
    Div(u32),
}

const SYSTICK_RANGE: u32 = 0x0100_0000;

impl Delay {
    /// Configures the system timer (SysTick) as a delay provider
    /// As access to the count register is possible without a reference, we can
    /// just drop it
    pub fn new(mut syst: SYST, clocks: Clocks) -> Delay {
        syst.set_clock_source(SystClkSource::Core);

        syst.set_reload(SYSTICK_RANGE - 1);
        syst.clear_current();
        syst.enable_counter();

        let scale = if clocks.sysclk().0 < 1_000_000 {
            Scale::Div(1_000_000 / clocks.sysclk().0)
        } else {
            Scale::Mult(clocks.sysclk().0 / 1_000_000)
        };

        Delay { scale }
    }
}

impl DelayMs<u32> for Delay {
    // At 48 MHz, calling delay_us with ms * 1_000 directly overflows at 0x15D868 (just over the max u16 value)
    fn delay_ms(&mut self, mut ms: u32) {
        const MAX_MS: u32 = 0x0000_FFFF;
        while ms != 0 {
            let current_ms = if ms <= MAX_MS { ms } else { MAX_MS };
            self.delay_us(current_ms * 1_000);
            ms -= current_ms;
        }
    }
}

impl DelayMs<u16> for Delay {
    fn delay_ms(&mut self, ms: u16) {
        self.delay_us(u32::from(ms) * 1_000);
    }
}

impl DelayMs<u8> for Delay {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(u16(ms));
    }
}

impl DelayUs<u32> for Delay {
    fn delay_us(&mut self, us: u32) {
        // The SysTick Reload Value register supports values between 1 and 0x00FFFFFF.
        // Here less than maximum is used so we have some play if there's a long running interrupt.
        const MAX_RVR: u32 = 0x007F_FFFF;

        let mut total_rvr = match self.scale {
            Scale::Div(x) => us / x,
            Scale::Mult(x) => us * x,
        };

        while total_rvr != 0 {
            let current_rvr = if total_rvr <= MAX_RVR {
                total_rvr
            } else {
                MAX_RVR
            };

            let start_count = SYST::get_current();
            total_rvr -= current_rvr;
            while (start_count.wrapping_sub(SYST::get_current()) % SYSTICK_RANGE) < current_rvr {}
        }
    }
}

impl DelayUs<u16> for Delay {
    fn delay_us(&mut self, us: u16) {
        self.delay_us(u32(us))
    }
}

impl DelayUs<u8> for Delay {
    fn delay_us(&mut self, us: u8) {
        self.delay_us(u32(us))
    }
}
