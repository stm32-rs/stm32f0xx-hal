//! # API for the Digital to Analog converter
//!
//! Currently only supports writing to the DR of the DAC,
//! just a basic one-shot conversion.
//!
//! ## Example
//! ``` no_run
//!#![deny(unused_imports)]
//!#![no_main]
//!#![no_std]
//!
//!use cortex_m;
//!use cortex_m_rt as rt;
//!use panic_halt;
//!
//!use stm32f0xx_hal as hal;
//!
//!use crate::hal::pac;
//!use crate::hal::prelude::*;
//!use crate::hal::dac::*;
//!
//!use rt::entry;
//!
//!enum Direction {
//!    Upcounting,
//!    Downcounting,
//!}
//!
//!#[entry]
//!fn main() -> ! {
//!    if let (Some(mut dp), Some(_cp)) = (pac::Peripherals::take(), cortex_m::Peripherals::take()) {
//!        cortex_m::interrupt::free(move |_| {
//!            let cs = unsafe { &bare_metal::CriticalSection::new() };
//!            let mut rcc = dp.RCC.configure().sysclk(8.mhz()).freeze(&mut dp.FLASH);
//!
//!            let gpioa = dp.GPIOA.split(&mut rcc);
//!            let mut dac = dac(dp.DAC, gpioa.pa4.into_analog(cs), &mut rcc);
//!
//!            dac.enable();
//!
//!            let mut dir = Direction::Upcounting;
//!            let mut val = 0;
//!
//!            dac.set_value(2058);
//!            cortex_m::asm::bkpt();
//!
//!            dac.set_value(4095);
//!            cortex_m::asm::bkpt();
//!
//!            loop {
//!                dac.set_value(val);
//!                match val {
//!                    0 => dir = Direction::Upcounting,
//!                    4095 => dir = Direction::Downcounting,
//!                    _ => (),
//!                };
//!
//!                match dir {
//!                    Direction::Upcounting => val += 1,
//!                    Direction::Downcounting => val -= 1,
//!                }
//!            }
//!        });
//!    }
//!
//!    loop {
//!        continue;
//!    }
//!}
//! ```
#![deny(unused_imports)]
use core::mem;

use crate::gpio::gpioa::{PA4, PA5};
use crate::gpio::Analog;
use crate::pac::DAC;
use crate::rcc::Rcc;

pub struct C1;
pub struct C2;

pub trait DacOut<V> {
    fn set_value(&mut self, val: V);
    fn get_value(&mut self) -> V;
}

pub trait DacPin {
    fn enable(&mut self);
}

pub trait Pins<DAC> {
    type Output;
}

impl Pins<DAC> for PA4<Analog> {
    type Output = C1;
}

impl Pins<DAC> for PA5<Analog> {
    type Output = C2;
}

impl Pins<DAC> for (PA4<Analog>, PA5<Analog>) {
    type Output = (C1, C2);
}

pub fn dac<PINS>(_dac: DAC, _pins: PINS, rcc: &mut Rcc) -> PINS::Output
where
    PINS: Pins<DAC>,
{
    // Enable DAC clocks
    rcc.regs.apb1enr.modify(|_, w| w.dacen().set_bit());

    // Reset DAC
    rcc.regs.apb1rstr.modify(|_, w| w.dacrst().set_bit());
    rcc.regs.apb1rstr.modify(|_, w| w.dacrst().clear_bit());

    unsafe { mem::MaybeUninit::uninit().assume_init() }
}

macro_rules! dac {
    ($CX:ident, $en:ident, $cen:ident, $cal_flag:ident, $trim:ident, $mode:ident, $dhrx:ident, $dac_dor:ident, $daccxdhr:ident) => {
        impl DacPin for $CX {
            fn enable(&mut self) {
                let dac = unsafe { &(*DAC::ptr()) };
                dac.cr.modify(|_, w| w.$en().set_bit());
            }
        }

        impl DacOut<u16> for $CX {
            fn set_value(&mut self, val: u16) {
                let dac = unsafe { &(*DAC::ptr()) };
                dac.$dhrx.write(|w| unsafe { w.bits(val as u32) });
            }

            fn get_value(&mut self) -> u16 {
                let dac = unsafe { &(*DAC::ptr()) };
                dac.$dac_dor.read().bits() as u16
            }
        }
    };
}

pub trait DacExt {
    fn constrain<PINS>(self, pins: PINS, rcc: &mut Rcc) -> PINS::Output
    where
        PINS: Pins<DAC>;
}

impl DacExt for DAC {
    fn constrain<PINS>(self, pins: PINS, rcc: &mut Rcc) -> PINS::Output
    where
        PINS: Pins<DAC>,
    {
        dac(self, pins, rcc)
    }
}

#[cfg(any(
    feature = "stm32f051",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
dac!(C1, en1, cen1, cal_flag1, otrim1, mode1, dhr12r1, dor1, dacc1dhr);

#[cfg(any(
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
dac!(C2, en2, cen2, cal_flag2, otrim2, mode2, dhr12r2, dor2, dacc2dhr);
