use crate::stm32::rcc::cfgr::SWW;
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
                hclk: None,
                pclk: None,
                sysclk: None,
                clock_src: SysClkSource::HSI,
            },
        }
    }
}

/// Constrained RCC peripheral
pub struct Rcc {
    pub cfgr: CFGR,
}

const HSI: u32 = 8_000_000; // Hz
const HSI48: u32 = 48_000_000; // Hz - (available on STM32F04x, STM32F07x and STM32F09x devices only)

#[allow(unused)]
enum SysClkSource {
    HSI,
    HSE(u32),
    HSI48,
}

#[allow(unused)]
pub struct CFGR {
    hclk: Option<u32>,
    pclk: Option<u32>,
    sysclk: Option<u32>,
    clock_src: SysClkSource,
}

#[cfg(feature = "device-selected")]
impl CFGR {
    pub fn hse<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.clock_src = SysClkSource::HSE(freq.into().0);
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
        // Default to lowest frequency clock on all systems.
        let sysclk = self.sysclk.unwrap_or(HSI);

        let r_sysclk; // The "real" sysclock value, calculated below
        let pllmul_bits;

        // Select clock source based on user input and capability
        // Highest selected frequency source available takes precedent.
        // For F04x, F07x, F09x parts, use HSI48 if requested.
        let src_clk_freq = match self.clock_src {
            SysClkSource::HSE(freq) => freq,
            SysClkSource::HSI48 => HSI48,
            _ => HSI,
        };

        // Pll check
        if sysclk == src_clk_freq {
            // Bypass pll if src clk and requested sysclk are the same, to save power.
            // The only reason to override this behaviour is if the sysclk source were HSI, and you
            // were running the USB off the PLL...
            pllmul_bits = None;
            r_sysclk = src_clk_freq;
        } else {
            let pllmul =
                (4 * self.sysclk.unwrap_or(src_clk_freq) + src_clk_freq) / src_clk_freq / 2;
            let pllmul = core::cmp::min(core::cmp::max(pllmul, 2), 16);
            r_sysclk = pllmul * src_clk_freq / 2;

            pllmul_bits = if pllmul == 2 {
                None
            } else {
                Some(pllmul as u8 - 2)
            };
        }

        let hpre_bits = self
            .hclk
            .map(|hclk| match r_sysclk / hclk {
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

        // Enable the requested clock
        match self.clock_src {
            SysClkSource::HSE(_) => {
                rcc.cr
                    .modify(|_, w| w.csson().on().hseon().on().hsebyp().not_bypassed());

                while !rcc.cr.read().hserdy().bit_is_set() {}
            }
            SysClkSource::HSI48 => {
                rcc.cr2.modify(|_, w| w.hsi48on().set_bit());
                while rcc.cr2.read().hsi48rdy().bit_is_clear() {}
            }
            SysClkSource::HSI => {
                rcc.cr.write(|w| w.hsion().set_bit());
                while rcc.cr.read().hsirdy().bit_is_clear() {}
            }
        };

        let rcc = unsafe { &*crate::stm32::RCC::ptr() };

        // Set up rcc based on above calculated configuration.

        // Enable PLL
        if let Some(pllmul_bits) = pllmul_bits {
            let pllsrc_bit: u8 = match self.clock_src {
                SysClkSource::HSI => 0b00,
                SysClkSource::HSI48 => 0b11,
                SysClkSource::HSE(_) => 0b01,
            };

            // Set PLL source and multiplier
            rcc.cfgr
                .write(|w| unsafe { w.pllsrc().bits(pllsrc_bit).pllmul().bits(pllmul_bits) });

            rcc.cr.write(|w| w.pllon().set_bit());
            while rcc.cr.read().pllrdy().bit_is_clear() {}

            rcc.cfgr.modify(|_, w| unsafe {
                w.ppre().bits(ppre_bits).hpre().bits(hpre_bits).sw().pll()
            });
        } else {
            let sw_var = match self.clock_src {
                SysClkSource::HSI => SWW::HSI,
                SysClkSource::HSI48 => SWW::HSI48,
                SysClkSource::HSE(_) => SWW::HSE,
            };

            // use HSI as source
            rcc.cfgr.write(|w| unsafe {
                w.ppre()
                    .bits(ppre_bits)
                    .hpre()
                    .bits(hpre_bits)
                    .sw()
                    .variant(sw_var)
            });
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
