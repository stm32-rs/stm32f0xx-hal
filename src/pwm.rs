use cast::{u16, u32};
use core::{marker::PhantomData, mem::MaybeUninit};

use crate::rcc::Rcc;

use crate::time::Hertz;
use embedded_hal as hal;

pub trait Pins<TIM, P> {
    const C1: bool = false;
    const C1N: bool = false;
    const C2: bool = false;
    const C2N: bool = false;
    const C3: bool = false;
    const C3N: bool = false;
    const C4: bool = false;
    type Channels;
}
use crate::timers::PinC1;
use crate::timers::PinC1N;
use crate::timers::PinC2;
use crate::timers::PinC2N;
use crate::timers::PinC3;
use crate::timers::PinC3N;
use crate::timers::PinC4;

pub struct C1;
pub struct C1N;
pub struct C2;
pub struct C2N;
pub struct C3;
pub struct C3N;
pub struct C4;

pub struct PwmChannels<TIM, CHANNELS> {
    _channel: PhantomData<CHANNELS>,
    _tim: PhantomData<TIM>,
}

macro_rules! pins_impl {
    ( $( ( $($PINX:ident),+ ), ( $($TRAIT:ident),+ ), ( $($ENCHX:ident),* ); )+ ) => {
        $(
            #[allow(unused_parens)]
            impl<TIM, $($PINX,)+> Pins<TIM, ($($ENCHX),+)> for ($($PINX),+)
            where
                $($PINX: $TRAIT<TIM>,)+
            {
                $(const $ENCHX: bool = true;)+
                type Channels = ($(PwmChannels<TIM, $ENCHX>),+);
            }
        )+
    };
}

pins_impl!(
    (P1, P2, P3, P4), (PinC1, PinC2, PinC3, PinC4), (C1, C2, C3, C4);
    (P1, P1N, P2, P2N, P3, P3N), (PinC1, PinC1N, PinC2, PinC2N, PinC3, PinC3N), (C1, C1N, C2, C2N, C3, C3N);
    (P1, P1N, P2, P2N), (PinC1, PinC1N, PinC2, PinC2N), (C1, C1N, C2, C2N);
    (P2, P2N, P3, P3N), (PinC2, PinC2N, PinC3, PinC3N), (C2, C2N, C3, C3N);
    (P1, P1N, P3, P3N), (PinC1, PinC1N, PinC3, PinC3N), (C1, C1N, C3, C3N);
    (P2, P3, P4), (PinC2, PinC3, PinC4), (C2, C3, C4);
    (P1, P3, P4), (PinC1, PinC3, PinC4), (C1, C3, C4);
    (P1, P2, P4), (PinC1, PinC2, PinC4), (C1, C2, C4);
    (P1, P2, P3), (PinC1, PinC2, PinC3), (C1, C2, C3);
    (P3, P4), (PinC3, PinC4), (C3, C4);
    (P2, P4), (PinC2, PinC4), (C2, C4);
    (P2, P3), (PinC2, PinC3), (C2, C3);
    (P1, P4), (PinC1, PinC4), (C1, C4);
    (P1, P3), (PinC1, PinC3), (C1, C3);
    (P1, P2), (PinC1, PinC2), (C1, C2);
    (P1, P1N), (PinC1, PinC1N), (C1, C1N);
    (P2, P2N), (PinC2, PinC2N), (C2, C2N);
    (P3, P3N), (PinC3, PinC3N), (C3, C3N);
    (P1), (PinC1), (C1);
    (P2), (PinC2), (C2);
    (P3), (PinC3), (C3);
    (P4), (PinC4), (C4);
    (P1N), (PinC1N), (C1N);
    (P2N), (PinC2N), (C2N);
    (P3N), (PinC3N), (C3N);
);

impl<TIM, P1: PinC1<TIM>, P2: PinC1<TIM>> PinC1<TIM> for (P1, P2) {}
impl<TIM, P1: PinC2<TIM>, P2: PinC2<TIM>> PinC2<TIM> for (P1, P2) {}
impl<TIM, P1: PinC3<TIM>, P2: PinC3<TIM>> PinC3<TIM> for (P1, P2) {}
impl<TIM, P1: PinC4<TIM>, P2: PinC4<TIM>> PinC4<TIM> for (P1, P2) {}

impl<TIM, P1: PinC1<TIM>, P2: PinC1<TIM>, P3: PinC1<TIM>> PinC1<TIM> for (P1, P2, P3) {}
impl<TIM, P1: PinC2<TIM>, P2: PinC2<TIM>, P3: PinC2<TIM>> PinC2<TIM> for (P1, P2, P3) {}
impl<TIM, P1: PinC3<TIM>, P2: PinC3<TIM>, P3: PinC3<TIM>> PinC3<TIM> for (P1, P2, P3) {}
impl<TIM, P1: PinC4<TIM>, P2: PinC4<TIM>, P3: PinC4<TIM>> PinC4<TIM> for (P1, P2, P3) {}

impl<TIM, P1: PinC1<TIM>, P2: PinC1<TIM>, P3: PinC1<TIM>, P4: PinC1<TIM>> PinC1<TIM>
    for (P1, P2, P3, P4)
{
}
impl<TIM, P1: PinC2<TIM>, P2: PinC2<TIM>, P3: PinC2<TIM>, P4: PinC2<TIM>> PinC2<TIM>
    for (P1, P2, P3, P4)
{
}
impl<TIM, P1: PinC3<TIM>, P2: PinC3<TIM>, P3: PinC3<TIM>, P4: PinC3<TIM>> PinC3<TIM>
    for (P1, P2, P3, P4)
{
}
impl<TIM, P1: PinC4<TIM>, P2: PinC4<TIM>, P3: PinC4<TIM>, P4: PinC4<TIM>> PinC4<TIM>
    for (P1, P2, P3, P4)
{
}

// the following timer have a main output switch, enable the automatic output
macro_rules! brk {
    (TIM1, $tim:ident) => {
        $tim.bdtr.modify(|_, w| w.aoe().set_bit());
    };
    (TIM15, $tim:ident) => {
        $tim.bdtr.modify(|_, w| w.aoe().set_bit());
    };
    (TIM16, $tim:ident) => {
        $tim.bdtr.modify(|_, w| w.aoe().set_bit());
    };
    (TIM17, $tim:ident) => {
        $tim.bdtr.modify(|_, w| w.aoe().set_bit());
    };
    ($_other:ident, $_tim:ident) => {};
}

// Timer with four output channels 16 Bit Timer
macro_rules! pwm_4_channels {
    ($($TIMX:ident: ($timX:ident, $timXen:ident, $timXrst:ident, $apbenr:ident, $apbrstr:ident),)+) => {
        $(
            pub fn $timX<P, PINS, T>(tim: $TIMX, _pins: PINS, rcc: &mut Rcc, freq: T) -> PINS::Channels
            where
                PINS: Pins<$TIMX, P>,
                T: Into<Hertz>,
            {
                // enable and reset peripheral to a clean slate state
                rcc.regs.$apbenr.modify(|_, w| w.$timXen().set_bit());
                rcc.regs.$apbrstr.modify(|_, w| w.$timXrst().set_bit());
                rcc.regs.$apbrstr.modify(|_, w| w.$timXrst().clear_bit());

                if PINS::C1 {
                    tim.ccmr1_output()
                        .modify(|_, w| w.oc1pe().set_bit().oc1m().pwm_mode1() );
                }
                if PINS::C2 {
                    tim.ccmr1_output()
                        .modify(|_, w| w.oc2pe().set_bit().oc2m().pwm_mode1() );
                }
                if PINS::C3 {
                    tim.ccmr2_output()
                        .modify(|_, w| w.oc3pe().set_bit().oc3m().pwm_mode1() );
                }
                if PINS::C4 {
                    tim.ccmr2_output()
                        .modify(|_, w| w.oc4pe().set_bit().oc4m().pwm_mode1() );
                }

                // If pclk is prescaled from hclk, the frequency fed into the timers is doubled
                let tclk = if rcc.clocks.hclk().0 == rcc.clocks.pclk().0 {
                    rcc.clocks.pclk().0
                } else {
                    rcc.clocks.pclk().0 * 2
                };
                let ticks = tclk / freq.into().0;

                let psc = u16((ticks - 1) / (1 << 16)).unwrap();
                tim.psc.write(|w| w.psc().bits(psc) );
                let arr = u16(ticks / u32(psc + 1)).unwrap();
                tim.arr.write(|w| unsafe { w.bits(u32(arr)) });

                // enable auto-reload preload
                tim.cr1.modify(|_, w| w.arpe().set_bit());

                // Trigger update event to load the registers
                tim.cr1.modify(|_, w| w.urs().set_bit());
                tim.egr.write(|w| w.ug().set_bit());
                tim.cr1.modify(|_, w| w.urs().clear_bit());

                brk!($TIMX, tim);
                tim.cr1.write(|w|
                    w.cms()
                        .bits(0b00)
                        .dir()
                        .clear_bit()
                        .opm()
                        .clear_bit()
                        .cen()
                        .set_bit()
                );
                //NOTE(unsafe) `PINS::Channels` is a ZST
                unsafe { MaybeUninit::uninit().assume_init() }
            }

            impl hal::PwmPin for PwmChannels<$TIMX, C1> {
                type Duty = u16;

                //NOTE(unsafe) atomic write with no side effects
                fn disable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc1e().clear_bit()) };
                }

                //NOTE(unsafe) atomic write with no side effects
                fn enable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc1e().set_bit()) };
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).ccr1().read().ccr().bits() as u16 }
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_max_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).arr.read().arr().bits() as u16 }
                }

                //NOTE(unsafe) atomic write with no side effects
                fn set_duty(&mut self, duty: u16) {
                    unsafe { (*$TIMX::ptr()).ccr1().write(|w| w.ccr().bits(duty.into())) }
                }
            }

            impl hal::PwmPin for PwmChannels<$TIMX, C2> {
                type Duty = u16;

                //NOTE(unsafe) atomic write with no side effects
                fn disable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc2e().clear_bit()) };
                }

                //NOTE(unsafe) atomic write with no side effects
                fn enable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc2e().set_bit()) };
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).ccr2().read().ccr().bits() as u16 }
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_max_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).arr.read().arr().bits() as u16 }
                }

                //NOTE(unsafe) atomic write with no side effects
                fn set_duty(&mut self, duty: u16) {
                    unsafe { (*$TIMX::ptr()).ccr2().write(|w| w.ccr().bits(duty.into())) }
                }
            }

            impl hal::PwmPin for PwmChannels<$TIMX, C3> {
                type Duty = u16;

                //NOTE(unsafe) atomic write with no side effects
                fn disable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc3e().clear_bit()) };
                }

                //NOTE(unsafe) atomic write with no side effects
                fn enable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc3e().set_bit()) };
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).ccr3().read().ccr().bits() as u16 }
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_max_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).arr.read().arr().bits() as u16 }
                }

                //NOTE(unsafe) atomic write with no side effects
                fn set_duty(&mut self, duty: u16) {
                    unsafe { (*$TIMX::ptr()).ccr3().write(|w| w.ccr().bits(duty.into())) }
                }
            }

            impl hal::PwmPin for PwmChannels<$TIMX, C4> {
                type Duty = u16;

                //NOTE(unsafe) atomic write with no side effects
                fn disable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc4e().clear_bit()) };
                }

                //NOTE(unsafe) atomic write with no side effects
                fn enable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc4e().set_bit()) };
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).ccr4().read().ccr().bits() as u16 }
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_max_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).arr.read().arr().bits() as u16 }
                }

                //NOTE(unsafe) atomic write with no side effects
                fn set_duty(&mut self, duty: u16) {
                    unsafe { (*$TIMX::ptr()).ccr4().write(|w| w.ccr().bits(duty.into())) }
                }
            }
        )+
    };
}

// Timer with four output channels three with complements 16 Bit Timer
macro_rules! pwm_4_channels_with_3_complementary_outputs {
    ($($TIMX:ident: ($timX:ident, $timXen:ident, $timXrst:ident, $apbenr:ident, $apbrstr:ident),)+) => {
        $(
            pub fn $timX<P, PINS, T>(tim: $TIMX, _pins: PINS, rcc: &mut Rcc, freq: T) -> PINS::Channels
            where
                PINS: Pins<$TIMX, P>,
                T: Into<Hertz>,
            {
                // enable and reset peripheral to a clean slate state
                rcc.regs.$apbenr.modify(|_, w| w.$timXen().set_bit());
                rcc.regs.$apbrstr.modify(|_, w| w.$timXrst().set_bit());
                rcc.regs.$apbrstr.modify(|_, w| w.$timXrst().clear_bit());

                if PINS::C1N | PINS::C1N | PINS::C1N {
                    tim.bdtr.modify(|_, w| w.ossr().set_bit());
                }
                if PINS::C1 {
                    tim.ccmr1_output()
                        .modify(|_, w| w.oc1pe().set_bit().oc1m().pwm_mode1() );
                }
                if PINS::C2 {
                    tim.ccmr1_output()
                        .modify(|_, w| w.oc2pe().set_bit().oc2m().pwm_mode1() );
                }
                if PINS::C3 {
                    tim.ccmr2_output()
                        .modify(|_, w| w.oc3pe().set_bit().oc3m().pwm_mode1() );
                }
                if PINS::C4 {
                    tim.ccmr2_output()
                        .modify(|_, w| w.oc4pe().set_bit().oc4m().pwm_mode1() );
                }

                // If pclk is prescaled from hclk, the frequency fed into the timers is doubled
                let tclk = if rcc.clocks.hclk().0 == rcc.clocks.pclk().0 {
                    rcc.clocks.pclk().0
                } else {
                    rcc.clocks.pclk().0 * 2
                };
                let ticks = tclk / freq.into().0;

                let psc = u16((ticks - 1) / (1 << 16)).unwrap();
                tim.psc.write(|w| w.psc().bits(psc) );
                let arr = u16(ticks / u32(psc + 1)).unwrap();
                tim.arr.write(|w| unsafe { w.bits(u32(arr)) });

                // enable auto-reload preload
                tim.cr1.modify(|_, w| w.arpe().set_bit());

                // Trigger update event to load the registers
                tim.cr1.modify(|_, w| w.urs().set_bit());
                tim.egr.write(|w| w.ug().set_bit());
                tim.cr1.modify(|_, w| w.urs().clear_bit());

                brk!($TIMX, tim);
                tim.cr1.write(|w|
                    w.cms()
                        .bits(0b00)
                        .dir()
                        .clear_bit()
                        .opm()
                        .clear_bit()
                        .cen()
                        .set_bit()
                );
                //NOTE(unsafe) `PINS::Channels` is a ZST
                unsafe { MaybeUninit::uninit().assume_init() }
            }

            impl hal::PwmPin for PwmChannels<$TIMX, C1> {
                type Duty = u16;

                //NOTE(unsafe) atomic write with no side effects
                fn disable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc1e().clear_bit()) };
                }

                //NOTE(unsafe) atomic write with no side effects
                fn enable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc1e().set_bit()) };
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).ccr1().read().ccr().bits() as u16 }
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_max_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).arr.read().arr().bits() as u16 }
                }

                //NOTE(unsafe) atomic write with no side effects
                fn set_duty(&mut self, duty: u16) {
                    unsafe { (*$TIMX::ptr()).ccr1().write(|w| w.ccr().bits(duty.into())) }
                }
            }

            impl hal::PwmPin for PwmChannels<$TIMX, C1N> {
                type Duty = u16;

                //NOTE(unsafe) atomic write with no side effects
                fn disable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc1ne().clear_bit()) };
                }

                //NOTE(unsafe) atomic write with no side effects
                fn enable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc1ne().set_bit()) };
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).ccr1().read().ccr().bits() as u16 }
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_max_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).arr.read().arr().bits() as u16 }
                }

                //NOTE(unsafe) atomic write with no side effects
                fn set_duty(&mut self, duty: u16) {
                    unsafe { (*$TIMX::ptr()).ccr1().write(|w| w.ccr().bits(duty.into())) }
                }
            }

            impl hal::PwmPin for PwmChannels<$TIMX, C2> {
                type Duty = u16;

                //NOTE(unsafe) atomic write with no side effects
                fn disable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc2e().clear_bit()) };
                }

                //NOTE(unsafe) atomic write with no side effects
                fn enable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc2e().set_bit()) };
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).ccr2().read().ccr().bits() as u16 }
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_max_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).arr.read().arr().bits() as u16 }
                }

                //NOTE(unsafe) atomic write with no side effects
                fn set_duty(&mut self, duty: u16) {
                    unsafe { (*$TIMX::ptr()).ccr2().write(|w| w.ccr().bits(duty.into())) }
                }
            }

            impl hal::PwmPin for PwmChannels<$TIMX, C2N> {
                type Duty = u16;

                //NOTE(unsafe) atomic write with no side effects
                fn disable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc2ne().clear_bit()) };
                }

                //NOTE(unsafe) atomic write with no side effects
                fn enable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc2ne().set_bit()) };
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).ccr2().read().ccr().bits() as u16 }
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_max_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).arr.read().arr().bits() as u16 }
                }

                //NOTE(unsafe) atomic write with no side effects
                fn set_duty(&mut self, duty: u16) {
                    unsafe { (*$TIMX::ptr()).ccr2().write(|w| w.ccr().bits(duty.into())) }
                }
            }

            impl hal::PwmPin for PwmChannels<$TIMX, C3> {
                type Duty = u16;

                //NOTE(unsafe) atomic write with no side effects
                fn disable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc3e().clear_bit()) };
                }

                //NOTE(unsafe) atomic write with no side effects
                fn enable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc3e().set_bit()) };
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).ccr3().read().ccr().bits() as u16 }
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_max_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).arr.read().arr().bits() as u16 }
                }

                //NOTE(unsafe) atomic write with no side effects
                fn set_duty(&mut self, duty: u16) {
                    unsafe { (*$TIMX::ptr()).ccr3().write(|w| w.ccr().bits(duty.into())) }
                }
            }

            impl hal::PwmPin for PwmChannels<$TIMX, C3N> {
                type Duty = u16;

                //NOTE(unsafe) atomic write with no side effects
                fn disable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc3ne().clear_bit()) };
                }

                //NOTE(unsafe) atomic write with no side effects
                fn enable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc3ne().set_bit()) };
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).ccr3().read().ccr().bits() as u16 }
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_max_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).arr.read().arr().bits() as u16 }
                }

                //NOTE(unsafe) atomic write with no side effects
                fn set_duty(&mut self, duty: u16) {
                    unsafe { (*$TIMX::ptr()).ccr3().write(|w| w.ccr().bits(duty.into())) }
                }
            }

            impl hal::PwmPin for PwmChannels<$TIMX, C4> {
                type Duty = u16;

                //NOTE(unsafe) atomic write with no side effects
                fn disable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc4e().clear_bit()) };
                }

                //NOTE(unsafe) atomic write with no side effects
                fn enable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc4e().set_bit()) };
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).ccr4().read().ccr().bits() as u16 }
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_max_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).arr.read().arr().bits() as u16 }
                }

                //NOTE(unsafe) atomic write with no side effects
                fn set_duty(&mut self, duty: u16) {
                    unsafe { (*$TIMX::ptr()).ccr4().write(|w| w.ccr().bits(duty.into())) }
                }
            }
        )+
    };
}

// General purpose timer with two output channels
#[cfg(any(
    feature = "stm32f030x8",
    feature = "stm32f030xc",
    feature = "stm32f051",
    feature = "stm32f058",
    feature = "stm32f070xb",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
macro_rules! pwm_2_channels {
    ($($TIMX:ident: ($timX:ident, $timXen:ident, $timXrst:ident, $apbenr:ident, $apbrstr:ident),)+) => {
        $(
            pub fn $timX<P, PINS, T>(tim: $TIMX, _pins: PINS, rcc: &mut Rcc, freq: T) -> PINS::Channels
            where
                PINS: Pins<$TIMX, P>,
                T: Into<Hertz>,
            {
                // enable and reset peripheral to a clean slate state
                rcc.regs.$apbenr.modify(|_, w| w.$timXen().set_bit());
                rcc.regs.$apbrstr.modify(|_, w| w.$timXrst().set_bit());
                rcc.regs.$apbrstr.modify(|_, w| w.$timXrst().clear_bit());

                if PINS::C1 {
                    tim.ccmr1_output().modify(|_, w| w.oc1pe().set_bit().oc1m().bits(6));
                }
                if PINS::C2 {
                    tim.ccmr1_output().modify(|_, w| w.oc2pe().set_bit().oc2m().bits(6));
                }

                // If pclk is prescaled from hclk, the frequency fed into the timers is doubled
                let tclk = if rcc.clocks.hclk().0 == rcc.clocks.pclk().0 {
                    rcc.clocks.pclk().0
                } else {
                    rcc.clocks.pclk().0 * 2
                };
                let ticks = tclk / freq.into().0;

                let psc = u16((ticks - 1) / (1 << 16)).unwrap();
                tim.psc.write(|w| w.psc().bits(psc) );
                let arr = u16(ticks / u32(psc + 1)).unwrap();
                tim.arr.write(|w| unsafe { w.bits(u32(arr)) });

                // enable auto-reload preload
                tim.cr1.modify(|_, w| w.arpe().set_bit());

                // Trigger update event to load the registers
                tim.cr1.modify(|_, w| w.urs().set_bit());
                tim.egr.write(|w| w.ug().set_bit());
                tim.cr1.modify(|_, w| w.urs().clear_bit());

                brk!($TIMX, tim);
                tim.cr1.write(|w|
                    w.opm()
                        .clear_bit()
                        .cen()
                        .set_bit()
                );
                //NOTE(unsafe) `PINS::Channels` is a ZST
                unsafe { MaybeUninit::uninit().assume_init() }
            }

            impl hal::PwmPin for PwmChannels<$TIMX, C1> {
                type Duty = u16;

                //NOTE(unsafe) atomic write with no side effects
                fn disable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc1e().clear_bit()) };
                }

                //NOTE(unsafe) atomic write with no side effects
                fn enable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc1e().set_bit()) };
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).ccr1().read().ccr().bits() as u16 }
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_max_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).arr.read().arr().bits() as u16 }
                }

                //NOTE(unsafe) atomic write with no side effects
                fn set_duty(&mut self, duty: u16) {
                    unsafe { (*$TIMX::ptr()).ccr1().write(|w| w.ccr().bits(duty.into())) }
                }
            }

            impl hal::PwmPin for PwmChannels<$TIMX, C2> {
                type Duty = u16;

                //NOTE(unsafe) atomic write with no side effects
                fn disable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc2e().clear_bit()) };
                }

                //NOTE(unsafe) atomic write with no side effects
                fn enable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc2e().set_bit()) };
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).ccr2().read().ccr().bits() as u16 }
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_max_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).arr.read().arr().bits() as u16 }
                }

                //NOTE(unsafe) atomic write with no side effects
                fn set_duty(&mut self, duty: u16) {
                    unsafe { (*$TIMX::ptr()).ccr2().write(|w| w.ccr().bits(duty.into())) }
                }
            }
        )+
    };
}

// General purpose timer with one output channel (TIM14)
macro_rules! pwm_1_channel {
    ($($TIMX:ident: ($timX:ident, $timXen:ident, $timXrst:ident, $apbenr:ident, $apbrstr:ident),)+) => {
        $(
            pub fn $timX<P, PINS, T>(tim: $TIMX, _pins: PINS, rcc: &mut Rcc, freq: T) -> PINS::Channels
            where
                PINS: Pins<$TIMX, P>,
                T: Into<Hertz>,
            {
                // enable and reset peripheral to a clean slate state
                rcc.regs.$apbenr.modify(|_, w| w.$timXen().set_bit());
                rcc.regs.$apbrstr.modify(|_, w| w.$timXrst().set_bit());
                rcc.regs.$apbrstr.modify(|_, w| w.$timXrst().clear_bit());

                if PINS::C1 {
                    tim.ccmr1_output().modify(|_, w| w.oc1pe().set_bit().oc1m().bits(6));
                }

                // If pclk is prescaled from hclk, the frequency fed into the timers is doubled
                let tclk = if rcc.clocks.hclk().0 == rcc.clocks.pclk().0 {
                    rcc.clocks.pclk().0
                } else {
                    rcc.clocks.pclk().0 * 2
                };
                let ticks = tclk / freq.into().0;

                let psc = u16((ticks - 1) / (1 << 16)).unwrap();
                tim.psc.write(|w| w.psc().bits(psc) );
                let arr = u16(ticks / u32(psc + 1)).unwrap();
                tim.arr.write(|w| unsafe { w.bits(u32(arr)) });

                // enable auto-reload preload
                tim.cr1.modify(|_, w| w.arpe().set_bit());

                // Trigger update event to load the registers
                tim.cr1.modify(|_, w| w.urs().set_bit());
                tim.egr.write(|w| w.ug().set_bit());
                tim.cr1.modify(|_, w| w.urs().clear_bit());

                brk!($TIMX, tim);
                tim.cr1.write(|w|
                    w.cen()
                        .set_bit()
                );
                //NOTE(unsafe) `PINS::Channels` is a ZST
                unsafe { MaybeUninit::uninit().assume_init() }
            }

            impl hal::PwmPin for PwmChannels<$TIMX, C1> {
                type Duty = u16;

                //NOTE(unsafe) atomic write with no side effects
                fn disable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc1e().clear_bit()) };
                }

                //NOTE(unsafe) atomic write with no side effects
                fn enable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc1e().set_bit()) };
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).ccr1().read().ccr().bits() as u16 }
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_max_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).arr.read().arr().bits() as u16 }
                }

                //NOTE(unsafe) atomic write with no side effects
                fn set_duty(&mut self, duty: u16) {
                    unsafe { (*$TIMX::ptr()).ccr1().write(|w| w.ccr().bits(duty.into())) }
                }
            }
        )+
    };
}

// General purpose timer with one output channel (TIM16/TIM17)
macro_rules! pwm_1_channel_with_complementary_outputs {
    ($($TIMX:ident: ($timX:ident, $timXen:ident, $timXrst:ident, $apbenr:ident, $apbrstr:ident),)+) => {
        $(
            pub fn $timX<P, PINS, T>(tim: $TIMX, _pins: PINS, rcc: &mut Rcc, freq: T) -> PINS::Channels
            where
                PINS: Pins<$TIMX, P>,
                T: Into<Hertz>,
            {
                // enable and reset peripheral to a clean slate state
                rcc.regs.$apbenr.modify(|_, w| w.$timXen().set_bit());
                rcc.regs.$apbrstr.modify(|_, w| w.$timXrst().set_bit());
                rcc.regs.$apbrstr.modify(|_, w| w.$timXrst().clear_bit());

                if PINS::C1 || PINS::C1N {
                    tim.ccmr1_output().modify(|_, w| w.oc1pe().set_bit().oc1m().bits(6));
                }

                // If pclk is prescaled from hclk, the frequency fed into the timers is doubled
                let tclk = if rcc.clocks.hclk().0 == rcc.clocks.pclk().0 {
                    rcc.clocks.pclk().0
                } else {
                    rcc.clocks.pclk().0 * 2
                };
                let ticks = tclk / freq.into().0;

                let psc = u16((ticks - 1) / (1 << 16)).unwrap();
                tim.psc.write(|w| w.psc().bits(psc) );
                let arr = u16(ticks / u32(psc + 1)).unwrap();
                tim.arr.write(|w| unsafe { w.bits(u32(arr)) });

                // enable auto-reload preload
                tim.cr1.modify(|_, w| w.arpe().set_bit());

                // Trigger update event to load the registers
                tim.cr1.modify(|_, w| w.urs().set_bit());
                tim.egr.write(|w| w.ug().set_bit());
                tim.cr1.modify(|_, w| w.urs().clear_bit());

                brk!($TIMX, tim);
                tim.cr1.write(|w|
                    w.opm()
                        .clear_bit()
                        .cen()
                        .set_bit()
                );

                //NOTE(unsafe) `PINS::Channels` is a ZST
                unsafe { MaybeUninit::uninit().assume_init() }
            }

            impl hal::PwmPin for PwmChannels<$TIMX, C1> {
                type Duty = u16;

                //NOTE(unsafe) atomic write with no side effects
                fn disable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc1e().clear_bit()) };
                }

                //NOTE(unsafe) atomic write with no side effects
                fn enable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc1e().set_bit()) };
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).ccr1().read().ccr().bits() as u16 }
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_max_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).arr.read().arr().bits() as u16 }
                }

                //NOTE(unsafe) atomic write with no side effects
                fn set_duty(&mut self, duty: u16) {
                    unsafe { (*$TIMX::ptr()).ccr1().write(|w| w.ccr().bits(duty.into())) }
                }
            }

            impl hal::PwmPin for PwmChannels<$TIMX, C1N> {
                type Duty = u16;

                //NOTE(unsafe) atomic write with no side effects
                fn disable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc1ne().clear_bit()) };
                }

                //NOTE(unsafe) atomic write with no side effects
                fn enable(&mut self) {
                    unsafe { (*($TIMX::ptr())).ccer.modify(|_, w| w.cc1ne().set_bit()) };
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).ccr1().read().ccr().bits() as u16 }
                }

                //NOTE(unsafe) atomic read with no side effects
                fn get_max_duty(&self) -> u16 {
                    unsafe { (*$TIMX::ptr()).arr.read().arr().bits() as u16 }
                }

                //NOTE(unsafe) atomic write with no side effects
                fn set_duty(&mut self, duty: u16) {
                    unsafe { (*$TIMX::ptr()).ccr1().write(|w| w.ccr().bits(duty.into())) }
                }
            }
        )+
    };
}

use crate::pac::*;

pwm_4_channels!(
    TIM3: (tim3, tim3en, tim3rst, apb1enr, apb1rstr),
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
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
pwm_4_channels!(
    TIM2: (tim2, tim2en, tim2rst, apb1enr, apb1rstr),
);

pwm_4_channels_with_3_complementary_outputs!(TIM1: (tim1, tim1en, tim1rst, apb2enr, apb2rstr),);
pwm_1_channel!(TIM14: (tim14, tim14en, tim14rst, apb1enr, apb1rstr),);

pwm_1_channel_with_complementary_outputs!(
    TIM16: (tim16, tim16en, tim16rst, apb2enr, apb2rstr),
    TIM17: (tim17, tim17en, tim17rst, apb2enr, apb2rstr),
);

#[cfg(any(
    feature = "stm32f030x8",
    feature = "stm32f030xc",
    feature = "stm32f051",
    feature = "stm32f058",
    feature = "stm32f070xb",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
pwm_2_channels! {
    TIM15: (tim15, tim15en, tim15rst, apb2enr, apb2rstr),
}
