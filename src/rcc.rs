use crate::time::Hertz;
use crate::stm32::rcc::cfgr::SWW;

pub enum ClockSource {
    HSI,
    HSE(u32)
}

/// Extension trait that constrains the `RCC` peripheral
pub trait RccExt {
    /// Constrains the `RCC` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> Rcc;
}

#[cfg(feature = "device-selected")]
impl RccExt for crate::stm32::RCC {
    fn constrain(self) -> Rcc {
        Rcc {
            cfgr: CFGR {
                hclk: None,
                pclk: None,
                sysclk: None,
                clock_src: ClockSource::HSI,
            },
        }
    }
}

/// Constrained RCC peripheral
pub struct Rcc {
    pub cfgr: CFGR,
}

#[allow(unused)]
const HSI: u32 = 8_000_000; // Hz

#[allow(unused)]
pub struct CFGR {
    hclk: Option<u32>,
    pclk: Option<u32>,
    sysclk: Option<u32>,

    clock_src: ClockSource,
}

#[cfg(feature = "device-selected")]
impl CFGR {
    pub fn hse<F>(mut self, freq: F) -> Self
        where
            F: Into<Hertz>,
    {
        self.clock_src = ClockSource::HSE(freq.into().0);
        self
    }

    pub fn hclk<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.hclk = Some(freq.into().0);
        self
    }

    pub fn pclk<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.pclk = Some(freq.into().0);
        self
    }

    pub fn sysclk<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.sysclk = Some(freq.into().0);
        self
    }

    pub fn freeze(self) -> Clocks {
        let base_freq = match self.clock_src {
            ClockSource::HSI => HSI,
            ClockSource::HSE(freq) => freq,
        };

        let pllmul = (4 * self.sysclk.unwrap_or(base_freq) + base_freq) / base_freq / 2;
        let pllmul = core::cmp::min(core::cmp::max(pllmul, 2), 16);
        let sysclk = pllmul * base_freq / 2;

        let pllmul_bits = if pllmul == 2 {
            None
        } else {
            Some(pllmul as u8 - 2)
        };

        let hpre_bits = self
            .hclk
            .map(|hclk| match sysclk / hclk {
                0 => unreachable!(),
                1 => 0b0111,
                2 => 0b1000,
                3...5 => 0b1001,
                6...11 => 0b1010,
                12...39 => 0b1011,
                40...95 => 0b1100,
                96...191 => 0b1101,
                192...383 => 0b1110,
                _ => 0b1111,
            })
            .unwrap_or(0b0111);

        let hclk = sysclk / (1 << (hpre_bits - 0b0111));

        let ppre_bits = self
            .pclk
            .map(|pclk| match hclk / pclk {
                0 => unreachable!(),
                1 => 0b011,
                2 => 0b100,
                3...5 => 0b101,
                6...11 => 0b110,
                _ => 0b111,
            })
            .unwrap_or(0b011);

        let ppre: u8 = 1 << (ppre_bits - 0b011);
        let pclk = hclk / cast::u32(ppre);

        let rcc = unsafe { &*crate::stm32::RCC::ptr() };

        // If we are using HSE, start it
        match self.clock_src {
            ClockSource::HSE(_) => {
                rcc.cr
                    .modify(|_, w| w.csson().on().hseon().on().hsebyp().not_bypassed());

                // Wait for HSE ready

                while !rcc.cr.read().hserdy().bit_is_set() {}
            },
            ClockSource::HSI => (),
        };

        // adjust flash wait states
        unsafe {
            let flash = &*crate::stm32::FLASH::ptr();
            flash.acr.write(|w| {
                w.latency().bits(if sysclk <= 24_000_000 {
                    0b000
                } else if sysclk <= 48_000_000 {
                    0b001
                } else {
                    0b010
                })
            })
        }

        if let Some(pllmul_bits) = pllmul_bits {
            // use PLL as source

            // If PLL is current source, switch to HSI
            if rcc.cfgr.read().sws().is_pll() {
                // Temporarily select HSI
                rcc.cfgr.write(|w| w.sw().hsi());

                // Wait for HSI enabled
                while !rcc.cfgr.read().sws().is_hsi() {}
            }

            // Disable the PLL
            rcc.cr.write(|w| w.pllon().off());
            // Wait for PLL ready to clear
            while rcc.cr.read().pllrdy().bit_is_set() {}

            let pllsrc_bit = match self.clock_src {
                ClockSource::HSI => 0,
                ClockSource::HSE(_) => 1,
            };

            rcc.cfgr.write(|w| unsafe {
                w
                    .pllsrc().bits(pllsrc_bit)
                    .pllmul().bits(pllmul_bits)
            });

            rcc.cr.write(|w| w.pllon().on());

            while rcc.cr.read().pllrdy().bit_is_clear() {}

            rcc.cfgr.modify(|_, w| unsafe {
                w.ppre().bits(ppre_bits).hpre().bits(hpre_bits).sw().pll()
            });
        } else {
            let sw_var = match self.clock_src {
                ClockSource::HSI => SWW::HSI,
                ClockSource::HSE(_) => SWW::HSE,
            };

            // use HSI as source
            rcc.cfgr
                .write(|w| unsafe { w.ppre().bits(ppre_bits).hpre().bits(hpre_bits).sw().variant(sw_var) });
        }

        Clocks {
            hclk: Hertz(hclk),
            pclk: Hertz(pclk),
            sysclk: Hertz(sysclk),
        }
    }
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy)]
pub struct Clocks {
    hclk: Hertz,
    pclk: Hertz,
    sysclk: Hertz,
}

impl Clocks {
    /// Returns the frequency of the AHB
    pub fn hclk(&self) -> Hertz {
        self.hclk
    }

    /// Returns the frequency of the APB
    pub fn pclk(&self) -> Hertz {
        self.pclk
    }

    /// Returns the system (core) frequency
    pub fn sysclk(&self) -> Hertz {
        self.sysclk
    }
}
