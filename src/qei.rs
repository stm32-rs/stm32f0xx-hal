//! API for using an integrated timer as a quadrature encoder
//!
//! # Example
//! Configuring TIM3 to count encoder counts for an encoder connected to pins pa6 and pa7
//! ``` no_run
//! use stm32f0xx_hal as hal;
//!
//! use crate::hal::pac;
//! use crate::hal::prelude::*;
//! use crate::hal::qei::*;
//!
//! cortex_m::interrupt::free(|cs| {
//!     let mut p = pac::Peripherals::take().unwrap();
//!     let mut rcc = p.RCC.configure().freeze(&mut p.FLASH);
//!
//!     let gpioa = p.GPIOA.split(&mut rcc);
//!     
//!     let qei_pins = (
//!         gpioa.pa6.into_alternate_af1(cs),
//!         gpioa.pa7.into_alternate_af1(cs)
//!     );
//!     let encoder = Qei::tim3(p.TIM3, qei_pins, &mut rcc);
//!
//!     loop {
//!         let direction = encoder.read_direction();
//!         let count = encoder.count();
//!     }
//! });
//! ```

use crate::pwm::Pins;
use crate::rcc::Rcc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The direction the quadrature encoder is counting
pub enum Direction {
    /// The encoder is counting down
    Downcounting,
    /// The encoder is counting up
    Upcounting,
}

/// Quadrature Encoder using an advanted timer periperal
pub struct Qei<TIMER> {
    timer: TIMER,
}

macro_rules! qei {
    ($($TIM: ident: ($tim:ident, $timXen:ident, $timXrst:ident, $apbenr:ident, $apbrstr:ident, $width:ident),)+) => {
        $(
            use crate::pac::$TIM;
            impl Qei<$TIM> {
                /// Configures a TIM peripheral as a quadrature encoder
                pub fn $tim<P, PINS>(tim: $TIM, _pins: PINS, rcc: &mut Rcc) -> Self
                where
                    PINS: Pins<$TIM, P>,
                {
                    // enable and reset peripherals to a clean slate state
                    rcc.regs.$apbenr.modify(|_, w| w.$timXen().set_bit());
                    rcc.regs.$apbrstr.modify(|_, w| w.$timXrst().set_bit());
                    rcc.regs.$apbrstr.modify(|_, w| w.$timXrst().clear_bit());

                    if PINS::C1 && PINS::C2 {
                        tim.ccmr1_input().modify(|_, w| w
                            .cc1s().ti1()
                            .cc2s().ti2()
                        );
                        tim.ccer.write(|w| w
                            .cc1p().set_bit()
                            .cc2p().set_bit()
                        );
                        tim.smcr.write(|w| w.sms().encoder_mode_3());
                    } else if PINS::C1 {
                        tim.ccmr1_input().modify(|_, w| w.cc1s().ti1());
                        tim.ccer.write(|w| w.cc1p().set_bit());
                        tim.smcr.write(|w| w.sms().encoder_mode_1());
                    } else if PINS::C2 {
                        tim.ccmr1_input().modify(|_, w| w.cc2s().ti2());
                        tim.ccer.write(|w| w.cc2p().set_bit());
                        tim.smcr.write(|w| w.sms().encoder_mode_2());
                    }

                    tim.arr.write(|w| w.arr().variant($width::MAX));
                    tim.cr1.write(|w| w.cen().set_bit());

                    Self {
                        timer: tim,
                    }
                }

                /// Read the direction the encoder is counting
                pub fn read_direction(&self) -> Direction {
                    match self.timer.cr1.read().dir().bit_is_set() {
                        true => Direction::Downcounting,
                        false => Direction::Upcounting,
                    }
                }

                /// Get the current count of the encoder
                pub fn count(&self) -> $width {
                    self.timer.cnt.read().cnt().bits()
                }
            }
        )+
    }
}

qei! {
    TIM3: (tim3, tim3en, tim3rst, apb1enr, apb1rstr, u16),
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
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098"
))]
qei! {
    TIM2: (tim2, tim2en, tim2rst, apb1enr, apb1rstr, u32),
}
