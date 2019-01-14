//! # API for the Analog to Digital converter
//!
//! Currently implements oneshot conversion with variable sampling times.
//! Also references for the internal temperature sense, voltage
//! reference and battery sense are provided.
//!
//! ## Example
//! ``` no_run
//! use stm32f0xx_hal as hal;
//!
//! use crate::hal::stm32;
//! use crate::hal::prelude::*;
//! use crate::hal::adc::Adc;
//!
//! cortex_m::interrupt::free(|cs| {
//!     let mut p = stm32::Peripherals::take().unwrap();
//!     let mut rcc = p.RCC.configure().freeze(&mut p.FLASH);
//!
//!     let gpioa = p.GPIOA.split(&mut rcc);
//!
//!     let mut led = gpioa.pa1.into_push_pull_pull_output(cs);
//!     let mut an_in = gpioa.pa0.into_analog(cs);
//!
//!     let mut delay = Delay::new(cp.SYST, &rcc);
//!
//!     let mut adc = Adc::new(p.ADC, &mut rcc);
//!
//!     loop {
//!         let val: u16 = adc.read(&mut an_in).unwrap();
//!         if val < ((1 << 8) - 1) {
//!             led.set_low();
//!         } else {
//!             led.set_high();
//!         }
//!         delay.delay_ms(50_u16);
//!     }
//! });
//! ```

const VREFCAL: *const u16 = 0x1FFF_F7BA as *const u16;
const VTEMPCAL30: *const u16 = 0x1FFF_F7B8 as *const u16;
const VTEMPCAL110: *const u16 = 0x1FFF_F7C2 as *const u16;
const VDD_CALIB: u16 = 3300;

use core::ptr;

use embedded_hal::{
    adc::{Channel, OneShot},
    blocking::delay::DelayUs,
};

use crate::{delay::Delay, gpio::*, rcc::Rcc, stm32};

/// Analog to Digital converter interface
pub struct Adc {
    rb: stm32::ADC,
    sample_time: AdcSampleTime,
    align: AdcAlign,
    precision: AdcPrecision,
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// ADC Sampling time
///
/// Options for the sampling time, each is T + 0.5 ADC clock cycles.
pub enum AdcSampleTime {
    /// 1.5 cycles sampling time
    T_1,
    /// 7.5 cycles sampling time
    T_7,
    /// 13.5 cycles sampling time
    T_13,
    /// 28.5 cycles sampling time
    T_28,
    /// 41.5 cycles sampling time
    T_41,
    /// 55.5 cycles sampling time
    T_55,
    /// 71.5 cycles sampling time
    T_71,
    /// 239.5 cycles sampling time
    T_239,
}

impl AdcSampleTime {
    fn write_bits(self, adc: &mut stm32::ADC) {
        unsafe {
            adc.smpr.write(|w| {
                w.smpr().bits(match self {
                    AdcSampleTime::T_1 => 0b000_u8,
                    AdcSampleTime::T_7 => 0b001_u8,
                    AdcSampleTime::T_13 => 0b010_u8,
                    AdcSampleTime::T_28 => 0b011_u8,
                    AdcSampleTime::T_41 => 0b100_u8,
                    AdcSampleTime::T_55 => 0b101_u8,
                    AdcSampleTime::T_71 => 0b110_u8,
                    AdcSampleTime::T_239 => 0b111_u8,
                })
            });
        }
    }

    /// Get the default sample time (currently 239.5 cycles)
    pub fn default() -> Self {
        AdcSampleTime::T_239
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// ADC Result Alignment
pub enum AdcAlign {
    /// Left aligned results (most significant bits)
    ///
    /// Results in all precisions returning a value in the range 0-65535.
    /// Depending on the precision the result will step by larger or smaller
    /// amounts.
    Left,
    /// Right aligned results (least significant bits)
    ///
    /// Results in all precisions returning values from 0-(2^bits-1) in
    /// steps of 1.
    Right,
    /// Left aligned results without correction of 6bit values.
    ///
    /// Returns left aligned results exactly as shown in RM0091 Fig.37.
    /// Where the values are left aligned within the u16, with the exception
    /// of 6 bit mode where the value is left aligned within the first byte of
    /// the u16.
    LeftAsRM,
}

impl AdcAlign {
    fn write_bits(self, adc: &mut stm32::ADC) {
        adc.cfgr1.write(|w| {
            w.align().bit(match self {
                AdcAlign::Left => true,
                AdcAlign::Right => false,
                AdcAlign::LeftAsRM => true,
            })
        });
    }

    /// Get the default alignment (currently right aligned)
    pub fn default() -> Self {
        AdcAlign::Right
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// ADC Sampling Precision
pub enum AdcPrecision {
    /// 12 bit precision
    B_12,
    /// 10 bit precision
    B_10,
    /// 8 bit precision
    B_8,
    /// 6 bit precision
    B_6,
}

impl AdcPrecision {
    fn write_bits(self, adc: &mut stm32::ADC) {
        unsafe {
            adc.cfgr1.write(|w| {
                w.res().bits(match self {
                    AdcPrecision::B_12 => 0b00_u8,
                    AdcPrecision::B_10 => 0b01_u8,
                    AdcPrecision::B_8 => 0b10_u8,
                    AdcPrecision::B_6 => 0b11_u8,
                })
            });
        }
    }

    /// Get the default precision (currently 12 bit precision)
    pub fn default() -> Self {
        AdcPrecision::B_12
    }
}

macro_rules! adc_pins {
    ($($pin:ty => $chan:expr),+ $(,)*) => {
        $(
            impl Channel<Adc> for $pin {
                type ID = u8;

                fn channel() -> u8 { $chan }
            }
        )+
    };
}

adc_pins!(
    gpioa::PA0<Analog> => 0_u8,
    gpioa::PA1<Analog> => 1_u8,
    gpioa::PA2<Analog> => 2_u8,
    gpioa::PA3<Analog> => 3_u8,
    gpioa::PA4<Analog> => 4_u8,
    gpioa::PA5<Analog> => 5_u8,
    gpioa::PA6<Analog> => 6_u8,
    gpioa::PA7<Analog> => 7_u8,
    gpiob::PB0<Analog> => 8_u8,
    gpiob::PB1<Analog> => 9_u8,
);

#[cfg(any(
    feature = "stm32f030",
    feature = "stm32f051",
    feature = "stm32f058",
    feature = "stm32f070",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f091"
))]
adc_pins!(
    gpioc::PC0<Analog> => 10_u8,
    gpioc::PC1<Analog> => 11_u8,
    gpioc::PC2<Analog> => 12_u8,
    gpioc::PC3<Analog> => 13_u8,
    gpioc::PC4<Analog> => 14_u8,
    gpioc::PC5<Analog> => 15_u8,
);

#[derive(Debug, Default)]
/// Internal temperature sensor (ADC Channel 16)
pub struct VTemp;

#[derive(Debug, Default)]
/// Internal voltage reference (ADC Channel 17)
pub struct VRef;

adc_pins!(
    VTemp => 16_u8,
    VRef  => 17_u8,
);

impl VTemp {
    /// Init a new VTemp
    pub fn new() -> Self {
        VTemp::default()
    }

    /// Enable the internal temperature sense, this has a wake up time
    /// t<sub>START</sub> which can be found in your micro's datasheet, you
    /// must wait at least that long after enabling before taking a reading.
    /// Remember to disable when not in use.
    pub fn enable(&mut self, adc: &mut Adc) {
        adc.rb.ccr.modify(|_, w| w.tsen().set_bit());
    }

    /// Disable the internal temperature sense.
    pub fn disable(&mut self, adc: &mut Adc) {
        adc.rb.ccr.modify(|_, w| w.tsen().clear_bit());
    }

    /// Checks if the temperature sensor is enabled, does not account for the
    /// t<sub>START</sub> time however.
    pub fn is_enabled(&self, adc: &Adc) -> bool {
        adc.rb.ccr.read().tsen().bit_is_set()
    }

    fn convert_temp(vtemp: u16, vdda: u16) -> i16 {
        let vtemp30_cal = i32::from(unsafe { ptr::read(VTEMPCAL30) }) * 100;
        let vtemp110_cal = i32::from(unsafe { ptr::read(VTEMPCAL110) }) * 100;

        let mut temperature = i32::from(vtemp) * 100;
        temperature = (temperature * (i32::from(vdda) / i32::from(VDD_CALIB))) - vtemp30_cal;
        temperature *= (110 - 30) * 100;
        temperature /= vtemp110_cal - vtemp30_cal;
        temperature += 3000;
        temperature as i16
    }

    /// Read the value of the internal temperature sensor and return the
    /// result in 100ths of a degree centigrade.
    ///
    /// Given a delay reference it will attempt to restrict to the
    /// minimum delay needed to ensure a 10 us t<sub>START</sub> value.
    /// Otherwise it will approximate the required delay using ADC reads.
    pub fn read(adc: &mut Adc, delay: Option<&mut Delay>) -> i16 {
        let mut vtemp = Self::new();
        let vtemp_preenable = vtemp.is_enabled(&adc);

        if !vtemp_preenable {
            vtemp.enable(adc);

            if let Some(dref) = delay {
                dref.delay_us(2_u16);
            } else {
                // Double read of vdda to allow sufficient startup time for the temp sensor
                VRef::read_vdda(adc);
            }
        }
        let vdda = VRef::read_vdda(adc);

        let prev_cfg = adc.default_cfg();

        let vtemp_val = adc.read(&mut vtemp).unwrap();

        if !vtemp_preenable {
            vtemp.disable(adc);
        }

        adc.restore_cfg(prev_cfg);

        Self::convert_temp(vtemp_val, vdda)
    }
}

impl VRef {
    /// Init a new VRef
    pub fn new() -> Self {
        VRef::default()
    }

    /// Enable the internal voltage reference, remember to disable when not in use.
    pub fn enable(&mut self, adc: &mut Adc) {
        adc.rb.ccr.modify(|_, w| w.vrefen().set_bit());
    }

    /// Disable the internal reference voltage.
    pub fn disable(&mut self, adc: &mut Adc) {
        adc.rb.ccr.modify(|_, w| w.vrefen().clear_bit());
    }

    /// Returns if the internal voltage reference is enabled.
    pub fn is_enabled(&self, adc: &Adc) -> bool {
        adc.rb.ccr.read().vrefen().bit_is_set()
    }

    /// Reads the value of VDDA in milli-volts
    pub fn read_vdda(adc: &mut Adc) -> u16 {
        let vrefint_cal = u32::from(unsafe { ptr::read(VREFCAL) });
        let mut vref = Self::new();

        let prev_cfg = adc.default_cfg();

        let vref_val: u32 = if vref.is_enabled(&adc) {
            adc.read(&mut vref).unwrap()
        } else {
            vref.enable(adc);

            let ret = adc.read(&mut vref).unwrap();

            vref.disable(adc);
            ret
        };

        adc.restore_cfg(prev_cfg);

        (u32::from(VDD_CALIB) * vrefint_cal / vref_val) as u16
    }
}

#[cfg(any(
    feature = "stm32f031",
    feature = "stm32f038",
    feature = "stm32f042",
    feature = "stm32f048",
    feature = "stm32f051",
    feature = "stm32f058",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f091",
))]
#[derive(Debug, Default)]
/// Battery reference voltage (ADC Channel 18)
pub struct VBat;

#[cfg(any(
    feature = "stm32f031",
    feature = "stm32f038",
    feature = "stm32f042",
    feature = "stm32f048",
    feature = "stm32f051",
    feature = "stm32f058",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f091",
))]
adc_pins!(
    VBat  => 18_u8,
);

#[cfg(any(
    feature = "stm32f031",
    feature = "stm32f038",
    feature = "stm32f042",
    feature = "stm32f048",
    feature = "stm32f051",
    feature = "stm32f058",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f091",
))]
impl VBat {
    /// Init a new VBat
    pub fn new() -> Self {
        VBat::default()
    }

    /// Enable the internal VBat sense, remember to disable when not in use
    /// as otherwise it will sap current from the VBat source.
    pub fn enable(&mut self, adc: &mut Adc) {
        adc.rb.ccr.modify(|_, w| w.vbaten().set_bit());
    }

    /// Disable the internal VBat sense.
    pub fn disable(&mut self, adc: &mut Adc) {
        adc.rb.ccr.modify(|_, w| w.vbaten().clear_bit());
    }

    /// Returns if the internal VBat sense is enabled
    pub fn is_enabled(&self, adc: &Adc) -> bool {
        adc.rb.ccr.read().vbaten().bit_is_set()
    }

    /// Reads the value of VBat in milli-volts
    pub fn read(adc: &mut Adc) -> u16 {
        let mut vbat = Self::new();

        let vbat_val: u16 = if vbat.is_enabled(&adc) {
            adc.read_abs_mv(&mut vbat)
        } else {
            vbat.enable(adc);

            let ret = adc.read_abs_mv(&mut vbat);

            vbat.disable(adc);
            ret
        };

        vbat_val * 2
    }
}

/// A stored ADC config, can be restored by using the `Adc::restore_cfg` method
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StoredConfig(AdcSampleTime, AdcAlign, AdcPrecision);

impl Adc {
    /// Init a new Adc
    ///
    /// Sets all configurable parameters to defaults, enables the HSI14 clock
    /// for the ADC if it is not already enabled and performs a boot time
    /// calibration. As such this method may take an appreciable time to run.
    pub fn new(adc: stm32::ADC, rcc: &mut Rcc) -> Self {
        let mut s = Self {
            rb: adc,
            sample_time: AdcSampleTime::default(),
            align: AdcAlign::default(),
            precision: AdcPrecision::default(),
        };
        s.select_clock(rcc);
        s.calibrate();
        s
    }

    /// Saves a copy of the current ADC config
    pub fn save_cfg(&mut self) -> StoredConfig {
        StoredConfig(self.sample_time, self.align, self.precision)
    }

    /// Restores a stored config
    pub fn restore_cfg(&mut self, cfg: StoredConfig) {
        self.sample_time = cfg.0;
        self.align = cfg.1;
        self.precision = cfg.2;
    }

    /// Resets the ADC config to default, returning the existing config as
    /// a stored config.
    pub fn default_cfg(&mut self) -> StoredConfig {
        let cfg = self.save_cfg();
        self.sample_time = AdcSampleTime::default();
        self.align = AdcAlign::default();
        self.precision = AdcPrecision::default();
        cfg
    }

    /// Set the Adc sampling time
    ///
    /// Options can be found in [AdcSampleTime](crate::adc::AdcSampleTime).
    pub fn set_sample_time(&mut self, t_samp: AdcSampleTime) {
        self.sample_time = t_samp;
    }

    /// Set the Adc result alignment
    ///
    /// Options can be found in [AdcAlign](crate::adc::AdcAlign).
    pub fn set_align(&mut self, align: AdcAlign) {
        self.align = align;
    }

    /// Set the Adc precision
    ///
    /// Options can be found in [AdcPrecision](crate::adc::AdcPrecision).
    pub fn set_precision(&mut self, precision: AdcPrecision) {
        self.precision = precision;
    }

    /// Returns the largest possible sample value for the current settings
    pub fn max_sample(&self) -> u16 {
        match self.align {
            AdcAlign::Left => u16::max_value(),
            AdcAlign::LeftAsRM => match self.precision {
                AdcPrecision::B_6 => u16::from(u8::max_value()),
                _ => u16::max_value(),
            },
            AdcAlign::Right => match self.precision {
                AdcPrecision::B_12 => (1 << 12) - 1,
                AdcPrecision::B_10 => (1 << 10) - 1,
                AdcPrecision::B_8 => (1 << 8) - 1,
                AdcPrecision::B_6 => (1 << 6) - 1,
            },
        }
    }

    /// Read the value of a channel and converts the result to milli-volts
    pub fn read_abs_mv<PIN: Channel<Adc, ID = u8>>(&mut self, pin: &mut PIN) -> u16 {
        let vdda = u32::from(VRef::read_vdda(self));
        let v: u32 = self.read(pin).unwrap();
        let max_samp = u32::from(self.max_sample());

        (v * vdda / max_samp) as u16
    }

    fn calibrate(&mut self) {
        /* Ensure that ADEN = 0 */
        if self.rb.cr.read().aden().bit_is_set() {
            /* Clear ADEN by setting ADDIS */
            self.rb.cr.modify(|_, w| w.addis().set_bit());
        }
        while self.rb.cr.read().aden().bit_is_set() {}

        /* Clear DMAEN */
        self.rb.cfgr1.modify(|_, w| w.dmaen().clear_bit());

        /* Start calibration by setting ADCAL */
        self.rb.cr.modify(|_, w| w.adcal().set_bit());

        /* Wait until calibration is finished and ADCAL = 0 */
        while self.rb.cr.read().adcal().bit_is_set() {}
    }

    fn select_clock(&mut self, rcc: &mut Rcc) {
        rcc.regs.apb2enr.modify(|_, w| w.adcen().set_bit());
        rcc.regs.cr2.write(|w| w.hsi14on().set_bit());
        while rcc.regs.cr2.read().hsi14rdy().bit_is_clear() {}
    }

    fn power_up(&mut self) {
        if self.rb.isr.read().adrdy().bit_is_set() {
            self.rb.isr.modify(|_, w| w.adrdy().clear_bit());
        }
        self.rb.cr.modify(|_, w| w.aden().set_bit());
        while self.rb.isr.read().adrdy().bit_is_clear() {}
    }

    fn power_down(&mut self) {
        self.rb.cr.modify(|_, w| w.adstp().set_bit());
        while self.rb.cr.read().adstp().bit_is_set() {}
        self.rb.cr.modify(|_, w| w.addis().set_bit());
        while self.rb.cr.read().aden().bit_is_set() {}
    }

    fn convert(&mut self, chan: u8) -> u16 {
        self.rb.chselr.write(|w| unsafe { w.bits(1_u32 << chan) });

        self.sample_time.write_bits(&mut self.rb);
        self.align.write_bits(&mut self.rb);
        self.precision.write_bits(&mut self.rb);

        self.rb.cr.modify(|_, w| w.adstart().set_bit());
        while self.rb.isr.read().eoc().bit_is_clear() {}

        let res = self.rb.dr.read().bits() as u16;
        if self.align == AdcAlign::Left && self.precision == AdcPrecision::B_6 {
            res << 8
        } else {
            res
        }
    }
}

impl<WORD, PIN> OneShot<Adc, WORD, PIN> for Adc
where
    WORD: From<u16>,
    PIN: Channel<Adc, ID = u8>,
{
    type Error = ();

    fn read(&mut self, _pin: &mut PIN) -> nb::Result<WORD, Self::Error> {
        self.power_up();
        let res = self.convert(PIN::channel());
        self.power_down();
        Ok(res.into())
    }
}
