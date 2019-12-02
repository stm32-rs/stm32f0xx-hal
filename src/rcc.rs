use crate::stm32::RCC;
use crate::time::Hertz;

/// Extension trait that sets up the `RCC` peripheral
pub trait RccExt {
    /// Configure the clocks of the RCC peripheral
    fn configure(self) -> CFGR;
}

impl RccExt for RCC {
    fn configure(self) -> CFGR {
        CFGR {
            hclk: None,
            pclk: None,
            sysclk: None,
            clock_src: SysClkSource::HSI,
            /// CRS is only available on devices with USB and HSI48
            #[cfg(any(
		feature = "stm32f031", // TODO: May be an SVD bug
		feature = "stm32f038", // TODO: May be an SVD bug
                feature = "stm32f042",
                feature = "stm32f048",
		feature = "stm32f051", // TODO: May be an SVD bug
		feature = "stm32f058", // TODO: May be an SVD bug
                feature = "stm32f072",
                feature = "stm32f078",
            ))]
            crs: None,
            rcc: self,
        }
    }
}

/// Constrained RCC peripheral
pub struct Rcc {
    pub clocks: Clocks,
    pub(crate) regs: RCC,
}

#[cfg(any(feature = "stm32f030", feature = "stm32f070",))]
mod inner {
    use crate::stm32::{rcc::cfgr::SWW, RCC};

    pub(super) const HSI: u32 = 8_000_000; // Hz

    pub(super) enum SysClkSource {
        HSI,
        HSE(u32),
    }

    pub(super) fn get_freq(c_src: &SysClkSource) -> u32 {
        // Select clock source based on user input and capability
        // Highest selected frequency source available takes precedent.
        match c_src {
            SysClkSource::HSE(freq) => *freq,
            _ => HSI,
        }
    }

    pub(super) fn enable_clock(rcc: &mut RCC, c_src: &SysClkSource) {
        // Enable the requested clock
        match c_src {
            SysClkSource::HSE(_) => {
                rcc.cr
                    .modify(|_, w| w.csson().on().hseon().on().hsebyp().not_bypassed());

                while !rcc.cr.read().hserdy().bit_is_set() {}
            }
            SysClkSource::HSI => {
                rcc.cr.write(|w| w.hsion().set_bit());
                while rcc.cr.read().hsirdy().bit_is_clear() {}
            }
        }
    }

    pub(super) fn enable_pll(
        rcc: &mut RCC,
        c_src: &SysClkSource,
        pllmul_bits: u8,
        ppre_bits: u8,
        hpre_bits: u8,
    ) {
        let pllsrc_bit: bool = match c_src {
            SysClkSource::HSI => false,
            SysClkSource::HSE(_) => true,
        };

        // Set PLL source and multiplier
        rcc.cfgr
            .modify(|_, w| unsafe { w.pllsrc().bit(pllsrc_bit).pllmul().bits(pllmul_bits) });

        rcc.cr.write(|w| w.pllon().set_bit());
        while rcc.cr.read().pllrdy().bit_is_clear() {}

        rcc.cfgr
            .modify(|_, w| unsafe { w.ppre().bits(ppre_bits).hpre().bits(hpre_bits).sw().pll() });
    }

    pub(super) fn get_sww(c_src: &SysClkSource) -> SWW {
        match c_src {
            SysClkSource::HSI => SWW::HSI,
            SysClkSource::HSE(_) => SWW::HSE,
        }
    }
}

#[cfg(any(
    feature = "stm32f031", // TODO: May be an SVD bug
    feature = "stm32f038", // TODO: May be an SVD bug
    feature = "stm32f042",
    feature = "stm32f048",
    feature = "stm32f051", // TODO: May be an SVD bug
    feature = "stm32f058", // TODO: May be an SVD bug
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
mod inner {
    use crate::stm32::{rcc::cfgr::SWW, RCC};

    pub(super) const HSI: u32 = 8_000_000; // Hz
    pub(super) const HSI48: u32 = 48_000_000; // Hz

    pub(super) enum SysClkSource {
        HSI,
        HSE(u32),
        HSI48,
    }

    pub(super) fn get_freq(c_src: &SysClkSource) -> u32 {
        // Select clock source based on user input and capability
        // Highest selected frequency source available takes precedent.
        match c_src {
            SysClkSource::HSE(freq) => *freq,
            SysClkSource::HSI48 => HSI48,
            _ => HSI,
        }
    }

    pub(super) fn enable_clock(rcc: &mut RCC, c_src: &SysClkSource) {
        // Enable the requested clock
        match c_src {
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
        }
    }

    pub(super) fn enable_pll(
        rcc: &mut RCC,
        c_src: &SysClkSource,
        pllmul_bits: u8,
        ppre_bits: u8,
        hpre_bits: u8,
    ) {
        let pllsrc_bit: u8 = match c_src {
            SysClkSource::HSI => 0b00,
            SysClkSource::HSI48 => 0b11,
            SysClkSource::HSE(_) => 0b01,
        };

        // Set PLL source and multiplier
        rcc.cfgr
            .modify(|_, w| w.pllsrc().bits(pllsrc_bit).pllmul().bits(pllmul_bits));

        rcc.cr.write(|w| w.pllon().set_bit());
        while rcc.cr.read().pllrdy().bit_is_clear() {}

        rcc.cfgr
            .modify(|_, w| unsafe { w.ppre().bits(ppre_bits).hpre().bits(hpre_bits).sw().pll() });
    }

    pub(super) fn get_sww(c_src: &SysClkSource) -> SWW {
        match c_src {
            SysClkSource::HSI => SWW::HSI,
            SysClkSource::HSI48 => SWW::HSI48,
            SysClkSource::HSE(_) => SWW::HSE,
        }
    }
}

use self::inner::SysClkSource;

pub struct CFGR {
    hclk: Option<u32>,
    pclk: Option<u32>,
    sysclk: Option<u32>,
    clock_src: SysClkSource,
    /// CRS is only available on devices with USB and HSI48
    #[cfg(any(
	feature = "stm32f031", // TODO: May be an SVD bug
	feature = "stm32f038", // TODO: May be an SVD bug
        feature = "stm32f042",
        feature = "stm32f048",
	feature = "stm32f051", // TODO: May be an SVD bug
	feature = "stm32f058", // TODO: May be an SVD bug
        feature = "stm32f072",
        feature = "stm32f078",
    ))]
    crs: Option<crate::stm32::CRS>,
    rcc: RCC,
}

impl CFGR {
    pub fn hse<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.clock_src = SysClkSource::HSE(freq.into().0);
        self
    }

    #[cfg(any(
        feature = "stm32f042",
        feature = "stm32f048",
        feature = "stm32f071",
        feature = "stm32f072",
        feature = "stm32f078",
        feature = "stm32f091",
        feature = "stm32f098",
    ))]
    pub fn hsi48(mut self) -> Self {
        self.clock_src = SysClkSource::HSI48;
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

    #[cfg(any(
        feature = "stm32f042",
        feature = "stm32f048",
        feature = "stm32f072",
        feature = "stm32f078",
    ))]
    pub fn enable_crs(mut self, crs: crate::stm32::CRS) -> Self {
        self.crs = Some(crs);
        self
    }

    pub fn freeze(mut self, flash: &mut crate::stm32::FLASH) -> Rcc {
        // Default to lowest frequency clock on all systems.
        let sysclk = self.sysclk.unwrap_or(self::inner::HSI);

        let r_sysclk; // The "real" sysclock value, calculated below
        let pllmul_bits;

        // Select clock source based on user input and capability
        // Highest selected frequency source available takes precedent.
        // For F04x, F07x, F09x parts, use HSI48 if requested.
        let src_clk_freq = self::inner::get_freq(&self.clock_src);

        // Pll check
        if sysclk == src_clk_freq {
            // Bypass pll if src clk and requested sysclk are the same, to save power.
            // The only reason to override this behaviour is if the sysclk source were HSI, and you
            // were running the USB off the PLL...
            pllmul_bits = None;
            r_sysclk = src_clk_freq;
        } else {
            // Select source frequency according to clock tree diagram in datasheet
            let src_clk_freq = match self.clock_src {
                // HSI frequency divide by two before PLL
                SysClkSource::HSI => src_clk_freq / 2,
                // HSE frequency divide by PREDIV before PLL
                // Until we don't change PREDIV value in CFGR2 it equals to one
                SysClkSource::HSE(_) => src_clk_freq,
            };

            let pllmul =
                (2 * self.sysclk.unwrap_or(src_clk_freq) + src_clk_freq) / src_clk_freq / 2;
            let pllmul = core::cmp::min(core::cmp::max(pllmul, 1), 16);
            r_sysclk = pllmul * src_clk_freq;

            pllmul_bits = if pllmul == 1 {
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
                3..=5 => 0b1001,
                6..=11 => 0b1010,
                12..=39 => 0b1011,
                40..=95 => 0b1100,
                96..=191 => 0b1101,
                192..=383 => 0b1110,
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
                3..=5 => 0b101,
                6..=11 => 0b110,
                _ => 0b111,
            })
            .unwrap_or(0b011);

        let ppre: u8 = 1 << (ppre_bits - 0b011);
        let pclk = hclk / cast::u32(ppre);

        // adjust flash wait states
        unsafe {
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
        self::inner::enable_clock(&mut self.rcc, &self.clock_src);

        // Set up rcc based on above calculated configuration.

        // Enable PLL
        if let Some(pllmul_bits) = pllmul_bits {
            self::inner::enable_pll(
                &mut self.rcc,
                &self.clock_src,
                pllmul_bits,
                ppre_bits,
                hpre_bits,
            );
        } else {
            let sw_var = self::inner::get_sww(&self.clock_src);

            // CRS is only available on devices with USB and HSI48
            #[cfg(any(
                feature = "stm32f042",
                feature = "stm32f048",
                feature = "stm32f072",
                feature = "stm32f078",
            ))]
            match self.crs {
                Some(crs) => {
                    self.rcc.apb1enr.modify(|_, w| w.crsen().set_bit());

                    // Initialize clock recovery
                    // Set autotrim enabled.
                    crs.cr.modify(|_, w| w.autotrimen().set_bit());
                    // Enable CR
                    crs.cr.modify(|_, w| w.cen().set_bit());
                }
                _ => {}
            }

            // use HSI as source
            self.rcc.cfgr.write(|w| unsafe {
                w.ppre()
                    .bits(ppre_bits)
                    .hpre()
                    .bits(hpre_bits)
                    .sw()
                    .variant(sw_var)
            });
        }
        Rcc {
            clocks: Clocks {
                hclk: Hertz(hclk),
                pclk: Hertz(pclk),
                sysclk: Hertz(sysclk),
            },
            regs: self.rcc,
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
