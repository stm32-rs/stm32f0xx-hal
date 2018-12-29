use core::marker::PhantomData;
use core::mem;

#[cfg(any(feature = "stm32f042"))]
use crate::stm32::TIM2;

use cast::{u16, u32};
use embedded_hal as hal;

use crate::gpio::{self, gpioa, gpiob, Alternate};
use crate::rcc::Clocks;
use crate::time::Hertz;

pub trait CH1<TIM> {}
pub trait CH2<TIM> {}
pub trait CH3<TIM> {}
pub trait CH4<TIM> {}

macro_rules! pwm_pins {
    ($($TIM:ident => {
        ch1 => [$($ch1:ty),+ $(,)*],
        ch2 => [$($ch2:ty),+ $(,)*],
        ch3 => [$($ch3:ty),+ $(,)*],
        ch4 => [$($ch4:ty),+ $(,)*],
    })+) => {
        $(
            $(
                impl CH1<crate::stm32::$TIM> for $ch1 {}
                impl Pins<$TIM> for $ch1 {
                    const C1: bool = true;
                    const C2: bool = false;
                    const C3: bool = false;
                    const C4: bool = false;
                    type Channels = Pwm<$TIM, C1>;
                }
            )+
            $(
                impl CH2<crate::stm32::$TIM> for $ch2 {}
                impl Pins<$TIM> for $ch2 {
                    const C1: bool = false;
                    const C2: bool = true;
                    const C3: bool = false;
                    const C4: bool = false;
                    type Channels = Pwm<$TIM, C2>;
                }
            )+
            $(
                impl CH3<crate::stm32::$TIM> for $ch3 {}
                impl Pins<$TIM> for $ch3 {
                    const C1: bool = false;
                    const C2: bool = false;
                    const C3: bool = true;
                    const C4: bool = false;
                    type Channels = Pwm<$TIM, C3>;
                }
            )+
            $(
                impl CH4<crate::stm32::$TIM> for $ch4 {}
                impl Pins<$TIM> for $ch4 {
                    const C1: bool = false;
                    const C2: bool = false;
                    const C3: bool = false;
                    const C4: bool = true;
                    type Channels = Pwm<$TIM, C4>;
                }
            )+
        )+
    }
}

#[cfg(any(feature = "stm32f042"))]
pwm_pins! {
    TIM2 => {
        ch1 => [gpioa::PA0<Alternate<gpio::AF2>>, gpioa::PA5<Alternate<gpio::AF2>>, gpioa::PA15<Alternate<gpio::AF2>>],
        ch2 => [gpioa::PA1<Alternate<gpio::AF2>>, gpiob::PB3<Alternate<gpio::AF2>>],
        ch3 => [gpioa::PA2<Alternate<gpio::AF2>>, gpiob::PB10<Alternate<gpio::AF2>>],
        ch4 => [gpioa::PA3<Alternate<gpio::AF2>>, gpiob::PB11<Alternate<gpio::AF2>>],
    }
}

pub trait Pins<TIM> {
    const C1: bool;
    const C2: bool;
    const C3: bool;
    const C4: bool;
    type Channels;
}

pub struct Pwm<TIM, CHANNEL> {
    _channel: PhantomData<CHANNEL>,
    _tim: PhantomData<TIM>,
}

pub struct C1;
pub struct C2;
pub struct C3;
pub struct C4;

#[allow(unused)]
macro_rules! tim {
    ($($TIM:ident: ($tim:ident, $timXen:ident, $timXrst:ident, $apbenr:ident, $apbrstr:ident),)+) => {
        $(
            pub fn $tim<PINS>(tim: $TIM, _pins: PINS, freq: Hertz, clocks: Clocks) -> PINS::Channels
            where
                PINS: Pins<$TIM>,
            {
                let rcc = unsafe { &(*crate::stm32::RCC::ptr()) };
                rcc.$apbenr.modify(|_, w| w.$timXen().set_bit());
                rcc.$apbrstr.modify(|_, w| w.$timXrst().set_bit());
                rcc.$apbrstr.modify(|_, w| w.$timXrst().clear_bit());

                if PINS::C1 {
                    tim.ccmr1_output
                        .modify(|_, w| unsafe { w.oc1pe().set_bit().oc1m().bits(6) });
                }

                if PINS::C2 {
                    tim.ccmr1_output
                        .modify(|_, w| unsafe { w.oc2pe().set_bit().oc2m().bits(6) });
                }

                if PINS::C3 {
                    tim.ccmr2_output
                        .modify(|_, w| unsafe { w.oc3pe().set_bit().oc3m().bits(6) });
                }

                if PINS::C4 {
                    tim.ccmr2_output
                        .modify(|_, w| unsafe { w.oc4pe().set_bit().oc4m().bits(6) });
                }

                let clk = clocks.pclk().0;
                let freq = freq.0;
                let ticks = clk / freq;
                let psc = u16(ticks / (1 << 16)).unwrap();
                tim.psc.write(|w| unsafe { w.psc().bits(psc) });
                let arr = u16(ticks / u32(psc + 1)).unwrap();
                tim.arr.write(|w| w.arr().bits(arr as u32));

                tim.cr1.write(|w| unsafe {
                    w.cms()
                        .bits(0b00)
                        .dir()
                        .clear_bit()
                        .opm()
                        .clear_bit()
                        .cen()
                        .set_bit()
                });

                unsafe { mem::uninitialized() }
            }

            impl hal::PwmPin for Pwm<$TIM, C1> {
                type Duty = u16;

                fn disable(&mut self) {
                    unsafe { (*$TIM::ptr()).ccer.write(|w| w.cc1e().clear_bit()) }
                }

                fn enable(&mut self) {
                    unsafe { (*$TIM::ptr()).ccer.write(|w| w.cc1e().set_bit()) }
                }

                fn get_duty(&self) -> Self::Duty {
                    unsafe { (*$TIM::ptr()).ccr1.read().ccr1().bits() as u16 }
                }

                fn get_max_duty(&self) -> Self::Duty {
                    unsafe { (*$TIM::ptr()).arr.read().arr().bits() as u16 }
                }

                fn set_duty(&mut self, duty: Self::Duty) {
                    unsafe { (*$TIM::ptr()).ccr1.write(|w| w.ccr1().bits(duty as u32)) }
                }
            }

            impl hal::PwmPin for Pwm<$TIM, C2> {
                type Duty = u16;

                fn disable(&mut self) {
                    unsafe { (*$TIM::ptr()).ccer.write(|w| w.cc2e().clear_bit()) }
                }

                fn enable(&mut self) {
                    unsafe { (*$TIM::ptr()).ccer.write(|w| w.cc2e().set_bit()) }
                }

                fn get_duty(&self) -> Self::Duty {
                    unsafe { (*$TIM::ptr()).ccr2.read().ccr2().bits() as u16 }
                }

                fn get_max_duty(&self) -> Self::Duty {
                    unsafe { (*$TIM::ptr()).arr.read().arr().bits() as u16 }
                }

                fn set_duty(&mut self, duty: Self::Duty) {
                    unsafe { (*$TIM::ptr()).ccr2.write(|w| w.ccr2().bits(duty as u32)) }
                }
            }

            impl hal::PwmPin for Pwm<$TIM, C3> {
                type Duty = u16;

                fn disable(&mut self) {
                    unsafe { (*$TIM::ptr()).ccer.write(|w| w.cc3e().clear_bit()) }
                }

                fn enable(&mut self) {
                    unsafe { (*$TIM::ptr()).ccer.write(|w| w.cc3e().set_bit()) }
                }

                fn get_duty(&self) -> Self::Duty {
                    unsafe { (*$TIM::ptr()).ccr3.read().ccr3().bits() as u16 }
                }

                fn get_max_duty(&self) -> Self::Duty {
                    unsafe { (*$TIM::ptr()).arr.read().arr().bits() as u16 }
                }

                fn set_duty(&mut self, duty: Self::Duty) {
                    unsafe { (*$TIM::ptr()).ccr3.write(|w| w.ccr3().bits(duty as u32)) }
                }
            }

            impl hal::PwmPin for Pwm<$TIM, C4> {
                type Duty = u16;

                fn disable(&mut self) {
                    unsafe { (*$TIM::ptr()).ccer.write(|w| w.cc4e().clear_bit()) }
                }

                fn enable(&mut self) {
                    unsafe { (*$TIM::ptr()).ccer.write(|w| w.cc4e().set_bit()) }
                }

                fn get_duty(&self) -> Self::Duty {
                    unsafe { (*$TIM::ptr()).ccr4.read().ccr4().bits() as u16 }
                }

                fn get_max_duty(&self) -> Self::Duty {
                    unsafe { (*$TIM::ptr()).arr.read().arr().bits() as u16 }
                }

                fn set_duty(&mut self, duty: Self::Duty) {
                    unsafe { (*$TIM::ptr()).ccr4.write(|w| w.ccr4().bits(duty as u32)) }
                }
            }
        )+
    }
}

#[cfg(any(feature = "stm32f042"))]
tim! {
    TIM2: (tim2, tim2en, tim2rst, apb1enr, apb1rstr),
}
