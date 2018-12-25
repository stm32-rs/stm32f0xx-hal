use crate::time::Hertz;

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
                hse: None,
                hclk: None,
                pclk: None,
                sysclk: None,
                clock_source: ClockSource::HSI,
            },
        }
    }
}

#[cfg(feature = "stm32f070")]
use stm32f0::stm32f0x0::rcc::cfgr::{SWW, PLLSRCW};

/// Constrained RCC peripheral
pub struct Rcc {
    pub cfgr: CFGR,
}

#[allow(unused)]
const HSI: u32 = 8_000_000; // Hz

#[allow(unused)]
pub enum ClockSource {
    /// Use internal clock as source
    HSI,
    /// Use External clock as source
    HSE,
}

#[allow(unused)]
pub struct CFGR {
    hse: Option<u32>,
    hclk: Option<u32>,
    pclk: Option<u32>,
    sysclk: Option<u32>,
    clock_source: ClockSource,
}

#[cfg(feature = "device-selected")]
impl CFGR {
    pub fn hse<F>(mut self, freq: F) -> Self
        where
            F: Into<Hertz>,
    {
        self.hse = Some(freq.into().0);
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

    pub fn clock_source(mut self, src: ClockSource) -> Self
    {
        self.clock_source = src;
        self
    }

    pub fn freeze(self) -> Clocks {
        let core_freq = match self.clock_source {
            ClockSource::HSI => HSI,
            ClockSource::HSE => self.hse.unwrap_or(HSI)
        };

        let pllmul = (4 * self.sysclk.unwrap_or(core_freq) + core_freq) / core_freq / 2;
        let pllmul = core::cmp::min(core::cmp::max(pllmul, 2), 16);
        let sysclk = pllmul * core_freq / 2;

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

        let rcc = unsafe { &*crate::stm32::RCC::ptr() };
        if let Some(pllmul_bits) = pllmul_bits {
            // use PLL as source

            let pll_src_variant = match self.clock_source {
                ClockSource::HSI => PLLSRCW::HSI_DIV_PREDIV,
                ClockSource::HSE => PLLSRCW::HSE_DIV_PREDIV,
            };

            rcc.cfgr.write(|w| unsafe { w.pllsrc().variant(pll_src_variant).pllmul().bits(pllmul_bits) });

            rcc.cr.write(|w| w.pllon().set_bit());

            while rcc.cr.read().pllrdy().bit_is_clear() {}

            rcc.cfgr.modify(|_, w| unsafe {
                w.ppre().bits(ppre_bits).hpre().bits(hpre_bits).sw().bits(2)
            });
        } else {
            let src_sw_variant = match self.clock_source {
                ClockSource::HSI => SWW::HSI,
                ClockSource::HSE => SWW::HSE,
            };

            // use HSI as source
            rcc.cfgr
                .write(|w| unsafe { w.ppre().bits(ppre_bits).hpre().bits(hpre_bits).sw().variant(src_sw_variant) });
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
