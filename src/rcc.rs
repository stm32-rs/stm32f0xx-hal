#[cfg(any(
    feature = "stm32f030",
    feature = "stm32f042",
    feature = "stm32f070"
    feature = "stm32f072",
))]
use crate::stm32::{FLASH, RCC};

use crate::time::Hertz;

/// Extension trait that constrains the `RCC` peripheral
pub trait RccExt {
    /// Constrains the `RCC` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> Rcc;
}

#[cfg(any(
    feature = "stm32f030",
    feature = "stm32f042",
    feature = "stm32f070"
    feature = "stm32f072",
))]
impl RccExt for RCC {
    fn constrain(self) -> Rcc {
        Rcc {
            cfgr: CFGR {
                hclk: None,
                pclk: None,
                sysclk: None,
                enable_hsi: None,
                enable_hsi14: None,
                // This does not work very well: #[cfg(any(feature = "stm32f042", feature = "stm32f072"))]
                enable_hsi48: None,
                enable_lsi: None,
                enable_pll: None,
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
#[allow(dead_code)] // Only used for ADC, but no configuration for ADC clock yet.
const HSI14: u32 = 14_000_000; // Hz - ADC clock.
// Can't do this anymore... #[cfg(any(feature = "stm32f042", feature = "stm32f072"))]
const HSI48: u32 = 48_000_000; // Hz - (available on STM32F04x, STM32F07x and STM32F09x devices only)

pub enum SysClkSource {
    HSI = 0b00,
    HSE = 0b01,
    PLL = 0b10,
    HSI48 = 0b11,
}

pub enum PllSource {
    HSI_2 = 0b00,
    HSI = 0b01,
    HSE = 0b10,
    HSI48 = 0b11,
}

#[allow(unused)]
pub struct CFGR {
    hclk: Option<u32>,
    pclk: Option<u32>,
    sysclk: Option<u32>,
    enable_hsi: Option<bool>,
    enable_hsi14: Option<bool>,
    // This does not work very well: #[cfg(any(feature = "stm32f042", feature = "stm32f072"))]
    enable_hsi48: Option<bool>,
    enable_lsi: Option<bool>,
    enable_pll: Option<bool>,
}

#[cfg(any(
    feature = "stm32f030",
    feature = "stm32f042",
    feature = "stm32f070"
))]
impl CFGR {
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

    pub fn enable_hsi(mut self, is_enabled: bool) -> Self {
        self.enable_hsi = Some(is_enabled);
        self
    }

    pub fn enable_hsi14(mut self, is_enabled: bool) -> Self {
        self.enable_hsi14 = Some(is_enabled);
        self
    }

    #[cfg(any(feature = "stm32f042", feature = "stm32f072"))]
    pub fn enable_hsi48(mut self, is_enabled: bool) -> Self {
        self.enable_hsi48 = Some(is_enabled);
        self
    }

    pub fn enable_lsi(mut self, is_enabled: bool) -> Self {
        self.enable_lsi = Some(is_enabled);
        self
    }

    pub fn enable_pll(mut self, is_enabled: bool) -> Self {
        self.enable_pll = Some(is_enabled);
        self
    }

    pub fn freeze(self) -> Clocks {
        // Default to lowest frequency clock on all systems.
        let sysclk = self.sysclk.unwrap_or(HSI);

        let r_sysclk; // The "real" sysclock value, calculated below
        let src_clk_freq; // Frequency of source clock for PLL and etc, HSI, or HSI48 on supported systems.
        let pllmul_bits;

        // Select clock source based on user input and capability
        // Highest selected frequency source available takes precident.
        // For F04x, F07x, F09x parts, use HSI48 if requested.
        if self.enable_hsi48.unwrap_or(false) { 
            // This does not work... #[cfg(any(feature = "stm32f042", feature = "stm32f072"))]
            src_clk_freq = HSI48; // Use HSI48 if requested and available.
        } else if self.enable_hsi.unwrap_or(true) {
            src_clk_freq = HSI; // HSI if requested, or by default.
        } else {
            src_clk_freq = HSI; // If no clock source is selclected use HSI.
        }
       
        // Pll check
        let enable_pll;
        if sysclk == src_clk_freq {
            // bypass pll if src clk and requested sysclk are the same, to save power.
            // The only reason to override this behaviour is if the sysclk source were HSI, and you
            // were running the USB off the PLL...
            enable_pll = false;
            pllmul_bits = None;
            r_sysclk = src_clk_freq;
        } else {
            let pllmul = (4 * self.sysclk.unwrap_or(src_clk_freq) + src_clk_freq) / src_clk_freq / 2;
            let pllmul = cmp::min(cmp::max(pllmul, 2), 16);
            r_sysclk = pllmul * src_clk_freq / 2;

            pllmul_bits = if pllmul == 2 {
                None
            } else {
                Some(pllmul as u8 - 2)
            };
            enable_pll = true; // Force PLL on.
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

        // adjust flash wait states
        unsafe {
            let flash = &*FLASH::ptr();
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

        // Set up rcc based on above calculated configuration.
        let rcc = unsafe { &*RCC::ptr() };

       
        // Enable requested clock sources
        // HSI
        if self.enable_hsi.unwrap_or(false) {
            rcc.cr.write(|w| w.hsion().set_bit());
            while rcc.cr.read().hsirdy().bit_is_clear() { }
        }
        // HSI14
        if self.enable_hsi14.unwrap_or(false) {
            rcc.cr2.modify(|_, w| w.hsi14on().set_bit());
            while rcc.cr2.read().hsi14rdy().bit_is_clear() {
                // nothing
            }
        }
        // HSI48
        if self.enable_hsi48.unwrap_or(false) {
            rcc.cr2.modify(|_, w| w.hsi48on().set_bit());
            while rcc.cr2.read().hsi48rdy().bit_is_clear() {
                // nothing
            }
        }

        // Enable PLL
        if let Some(pllmul_bits) = pllmul_bits {
            rcc.cfgr.write(|w| unsafe { w.pllmul().bits(pllmul_bits) });

            // Set PLL source based on configuration.
            if self.enable_hsi48.unwrap_or(false) {
                rcc.cfgr.modify(|_, w| w.pllsrc().bits(PllSource::HSI48 as u8));
            } else if self.enable_hsi.unwrap_or(true) {
                rcc.cfgr.modify(|_, w| w.pllsrc().bits(PllSource::HSI_2 as u8));
            } else {
                rcc.cfgr.modify(|_, w| w.pllsrc().bits(PllSource::HSI_2 as u8));
            }

            rcc.cr.write(|w| w.pllon().set_bit());
            while rcc.cr.read().pllrdy().bit_is_clear() { }

            rcc.cfgr.modify(|_, w| unsafe {
                w.ppre()
                    .bits(ppre_bits)
                    .hpre()
                    .bits(hpre_bits)
                    .sw()
                    .bits(SysClkSource::PLL as u8)
            });

        } else { // No PLL required.
            // Setup requested clocks.
            if self.enable_hsi48.unwrap_or(false) {
                rcc.cfgr.modify(|_, w| unsafe { 
                    w.ppre().bits(ppre_bits)
                     .hpre().bits(hpre_bits)
                     .sw().bits(SysClkSource::HSI48 as u8)
                });
            } else if self.enable_hsi.unwrap_or(true) {
                rcc.cfgr.modify(|_, w| unsafe { 
                    w.ppre().bits(ppre_bits)
                     .hpre().bits(hpre_bits)
                     .sw().bits(SysClkSource::HSI as u8)
                });
            } else { // Default to HSI
                rcc.cfgr.modify(|_, w| unsafe {
                    w.ppre().bits(ppre_bits)
                     .hpre().bits(hpre_bits)
                     .sw().bits(SysClkSource::HSI as u8)
                });
            }
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
