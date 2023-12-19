use embedded_hal::prelude::_embedded_hal_watchdog_Watchdog;

/// Bits per second
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Bps(pub u32);

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Hertz(pub u32);

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct KiloHertz(pub u32);

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct MegaHertz(pub u32);

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct MicroSecond(pub u32);

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct MilliSecond(pub u32);

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Second(pub u32);

/// Extension trait that adds convenience methods to the `u32` type
pub trait U32Ext {
    /// Wrap in `Bps`
    fn bps(self) -> Bps;

    /// Wrap in `Hertz`
    fn hz(self) -> Hertz;

    /// Wrap in `KiloHertz`
    fn khz(self) -> KiloHertz;

    /// Wrap in `MegaHertz`
    fn mhz(self) -> MegaHertz;

    /// Wrap in `MicroSecond`
    fn us(self) -> MicroSecond;

    /// Wrap in `MilliSecond`
    fn ms(self) -> MilliSecond;

    /// Wrap in `Second`
    fn seconds(self) -> Second;
}

impl U32Ext for u32 {
    fn bps(self) -> Bps {
        Bps(self)
    }

    fn hz(self) -> Hertz {
        Hertz(self)
    }

    fn khz(self) -> KiloHertz {
        KiloHertz(self)
    }

    fn mhz(self) -> MegaHertz {
        MegaHertz(self)
    }

    fn us(self) -> MicroSecond {
        MicroSecond(self)
    }

    fn ms(self) -> MilliSecond {
        MilliSecond(self)
    }

    fn seconds(self) -> Second {
        Second(self)
    }
}

impl From<KiloHertz> for Hertz {
    fn from(khz: KiloHertz) -> Self {
        Hertz(khz.0 * 1_000)
    }
}

impl From<MegaHertz> for Hertz {
    fn from(mhz: MegaHertz) -> Self {
        Hertz(mhz.0 * 1_000_000)
    }
}

impl From<MegaHertz> for KiloHertz {
    fn from(mhz: MegaHertz) -> Self {
        KiloHertz(mhz.0 * 1_000)
    }
}

impl Hertz {
    pub fn duration(self, cycles: u32) -> MicroSecond {
        let cycles = cycles as u64;
        let clk = self.0 as u64;
        let us = cycles.saturating_mul(1_000_000_u64) / clk;
        MicroSecond(us as u32)
    }
}

impl MicroSecond {
    pub fn cycles(self, clk: Hertz) -> u32 {
        assert!(self.0 > 0);
        let clk = clk.0 as u64;
        let period = self.0 as u64;
        let cycles = clk.saturating_mul(period) / 1_000_000_u64;
        cycles as u32
    }
}

impl From<Second> for MicroSecond {
    fn from(period: Second) -> MicroSecond {
        MicroSecond(period.0 * 1_000_000)
    }
}

impl From<Second> for MilliSecond {
    fn from(period: Second) -> MilliSecond {
        MilliSecond(period.0 * 1_000)
    }
}

impl From<MicroSecond> for MilliSecond {
    fn from(period: MicroSecond) -> MilliSecond {
        MilliSecond(period.0 * 1_000)
    }
}

impl From<Hertz> for MicroSecond {
    fn from(freq: Hertz) -> MicroSecond {
        assert!(freq.0 <= 1_000_000);
        MicroSecond(1_000_000 / freq.0)
    }
}
