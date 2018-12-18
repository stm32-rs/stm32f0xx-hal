use core::cmp;

use cast::u32;
use stm32::{FLASH, RCC};
use cortex_m_semihosting::{debug, hprintln};

use time::Hertz;

/// Extension trait that constrains the `RCC` peripheral
pub trait RccExt {
    /// Constrains the `RCC` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> Rcc;
}

impl RccExt for RCC {
    fn constrain(self) -> Rcc {
        Rcc {
            cfgr: CFGR {
                hclk: None,
                pclk: None,
                sysclk: None,
                enable_hsi: None,
                enable_hsi14: None,
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

const HSI: u32 = 8_000_000; // Hz
#[allow(dead_code)]
const HSI14: u32 = 14_000_000; // Hz - ADC clock.
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

pub struct CFGR {
    hclk:           Option<u32>,
    pclk:           Option<u32>,
    sysclk:         Option<u32>,
    enable_hsi:     Option<bool>,
    enable_hsi14:   Option<bool>,
    enable_hsi48:   Option<bool>,
    enable_lsi:     Option<bool>,
    enable_pll:     Option<bool>,
}

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

    pub fn enable_hsi(mut self, is_enabled: bool) -> Self
    {
        self.enable_hsi = Some(is_enabled);
        self
    }

    pub fn enable_hsi14(mut self, is_enabled: bool) -> Self
    {
        self.enable_hsi14 = Some(is_enabled);
        self
    }

    pub fn enable_hsi48(mut self, is_enabled: bool) -> Self
    {
        self.enable_hsi48 = Some(is_enabled);
        self
    }

    pub fn enable_lsi(mut self, is_enabled: bool) -> Self
    {
        self.enable_lsi = Some(is_enabled);
        self
    }

    pub fn enable_pll(mut self, is_enabled: bool) -> Self
    {
        self.enable_pll = Some(is_enabled);
        self
    }

    pub fn freeze(self) -> Clocks {
        let sysclk = self.sysclk.unwrap_or(HSI);

        // For F04x, F07x, F09x parts, use HSI48 for sysclk if someone requests sysclk == HSI48;
        let r_sysclk; // The "real" sysclock value, calculated below
        let pllmul_bits;
        if sysclk == HSI48 {
            pllmul_bits = None;
            r_sysclk = HSI48;
        } else {
            let pllmul = (4 * self.sysclk.unwrap_or(HSI) + HSI) / HSI / 2;
            let pllmul = cmp::min(cmp::max(pllmul, 2), 16);
            r_sysclk = pllmul * HSI / 2;

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
        let pclk = hclk / u32(ppre);

        hprintln!(
            "H: {:x} HP: {:x} P: {:x} PP: {:x} PP2: {:x}",
            hclk,
            hpre_bits,
            pclk,
            ppre_bits,
            ppre
        )
        .unwrap();

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

        let rcc = unsafe { &*RCC::ptr() };
        if let Some(pllmul_bits) = pllmul_bits {
            // use PLL as source

            rcc.cfgr.write(|w| unsafe { w.pllmul().bits(pllmul_bits) });

            rcc.cr.write(|w| w.pllon().set_bit());

            while rcc.cr.read().pllrdy().bit_is_clear() {}

            rcc.cfgr.modify(|_, w| unsafe {
                w.ppre()
                    .bits(ppre_bits)
                    .hpre()
                    .bits(hpre_bits)
                    .sw()
                    .bits(SysClkSource::PLL as u8)
            });
            hprintln!("PLL: {:x}", rcc.cfgr.read().bits() as u16).unwrap();
        } else if r_sysclk == HSI48 {
            // Enable HSI48
            rcc.cr2.modify(|_, w| w.hsi48on().set_bit());
            while ! rcc.cr2.read().hsi48rdy().bit_is_set()
            { // nothing 
            }

            // Set HSI48 as system clock.
            rcc.cfgr.modify(|_, w| unsafe {
                w.ppre()
                    //.bits(ppre_bits)
                    .bits(0)
                    .hpre()
                    //.bits(hpre_bits)
                    .bits(0)
                    .sw()
                    .bits(SysClkSource::HSI48 as u8)
            });
            hprintln!("HSI48: {:x}", rcc.cfgr.read().bits() as u32).unwrap();
        } else {
            // use HSI as source
            rcc.cfgr
                .write(|w| unsafe { w.ppre().bits(ppre_bits).hpre().bits(hpre_bits).sw().bits(0) });
            hprintln!("HSI: {:x}", rcc.cfgr.read().bits() as u16).unwrap();
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
