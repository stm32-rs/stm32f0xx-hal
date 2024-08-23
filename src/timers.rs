//! API for the integrated timers
//!
//! This only implements basic functions, a lot of things are missing
//!
//! # Example
//! Blink the led with 1Hz
//! ``` no_run
//! use stm32f0xx_hal as hal;
//!
//! use crate::hal::pac;
//! use crate::hal::prelude::*;
//! use crate::hal::time::*;
//! use crate::hal::timers::*;
//! use nb::block;
//!
//! cortex_m::interrupt::free(|_| {
//!     let cs = unsafe { &bare_metal::CriticalSection::new() };
//!     let mut p = pac::Peripherals::take().unwrap();
//!     let mut rcc = p.RCC.configure().freeze(&mut p.FLASH);
//!
//!     let gpioa = p.GPIOA.split(&mut rcc);
//!
//!     let mut led = gpioa.pa1.into_push_pull_pull_output(cs);
//!
//!     let mut timer = Timer::tim1(p.TIM1, Hertz(1), &mut rcc);
//!     loop {
//!         led.toggle();
//!         block!(timer.wait()).ok();
//!     }
//! });
//! ```
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;

use crate::rcc::{Clocks, Rcc};

use crate::time::Hertz;
use embedded_hal::timer::{CountDown, Periodic};
use void::Void;

/// Hardware timers
pub struct Timer<TIM> {
    clocks: Clocks,
    tim: TIM,
}

/// Interrupt events
pub enum Event {
    /// Timer timed out / count down ended
    TimeOut,
}

impl Timer<SYST> {
    /// Configures the SYST clock as a periodic count down timer
    pub fn syst<T>(mut syst: SYST, timeout: T, rcc: &Rcc) -> Self
    where
        T: Into<Hertz>,
    {
        syst.set_clock_source(SystClkSource::Core);
        let mut timer = Timer {
            tim: syst,
            clocks: rcc.clocks,
        };
        timer.start(timeout);
        timer
    }

    /// Starts listening for an `event`
    pub fn listen(&mut self, event: &Event) {
        match event {
            Event::TimeOut => self.tim.enable_interrupt(),
        }
    }

    /// Stops listening for an `event`
    pub fn unlisten(&mut self, event: &Event) {
        match event {
            Event::TimeOut => self.tim.disable_interrupt(),
        }
    }
}

/// Use the systick as a timer
///
/// Be aware that intervals less than 4 Hertz may not function properly
impl CountDown for Timer<SYST> {
    type Time = Hertz;

    /// Start the timer with a `timeout`
    fn start<T>(&mut self, timeout: T)
    where
        T: Into<Hertz>,
    {
        let rvr = self.clocks.sysclk().0 / timeout.into().0 - 1;

        assert!(rvr < (1 << 24));

        self.tim.set_reload(rvr);
        self.tim.clear_current();
        self.tim.enable_counter();
    }

    /// Return `Ok` if the timer has wrapped
    /// Automatically clears the flag and restarts the time
    fn wait(&mut self) -> nb::Result<(), Void> {
        if self.tim.has_wrapped() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl Periodic for Timer<SYST> {}

macro_rules! timers {
    ($($TIM:ident: ($tim:ident, $timXen:ident, $timXrst:ident, $apbenr:ident, $apbrstr:ident),)+) => {
        $(
            use crate::pac::$TIM;
            impl Timer<$TIM> {
                // XXX(why not name this `new`?) bummer: constructors need to have different names
                // even if the `$TIM` are non overlapping (compare to the `free` function below
                // which just works)
                /// Configures a TIM peripheral as a periodic count down timer
                pub fn $tim<T>(tim: $TIM, timeout: T, rcc: &mut Rcc) -> Self
                where
                    T: Into<Hertz>,
                {
                    // enable and reset peripheral to a clean slate state
                    rcc.regs.$apbenr.modify(|_, w| w.$timXen().set_bit());
                    rcc.regs.$apbrstr.modify(|_, w| w.$timXrst().set_bit());
                    rcc.regs.$apbrstr.modify(|_, w| w.$timXrst().clear_bit());

                    let mut timer = Timer {
                        clocks: rcc.clocks,
                        tim,
                    };
                    timer.start(timeout);

                    timer
                }

                /// Starts listening for an `event`
                pub fn listen(&mut self, event: Event) {
                    match event {
                        Event::TimeOut => {
                            // Enable update event interrupt
                            self.tim.dier.write(|w| w.uie().set_bit());
                        }
                    }
                }

                /// Stops listening for an `event`
                pub fn unlisten(&mut self, event: Event) {
                    match event {
                        Event::TimeOut => {
                            // Enable update event interrupt
                            self.tim.dier.write(|w| w.uie().clear_bit());
                        }
                    }
                }

                /// Releases the TIM peripheral
                pub fn release(self) -> $TIM {
                    let rcc = unsafe { &(*crate::pac::RCC::ptr()) };
                    // Pause counter
                    self.tim.cr1.modify(|_, w| w.cen().clear_bit());
                    // Disable timer
                    rcc.$apbenr.modify(|_, w| w.$timXen().clear_bit());
                    self.tim
                }

                /// Clears interrupt flag
                pub fn clear_irq(&mut self) {
                    self.tim.sr.modify(|_, w| w.uif().clear_bit());
                }
            }

            impl CountDown for Timer<$TIM> {
                type Time = Hertz;

                /// Start the timer with a `timeout`
                fn start<T>(&mut self, timeout: T)
                where
                    T: Into<Hertz>,
                {
                    // pause
                    self.tim.cr1.modify(|_, w| w.cen().clear_bit());
                    // restart counter
                    self.tim.cnt.reset();

                    let frequency = timeout.into().0;
                    // If pclk is prescaled from hclk, the frequency fed into the timers is doubled
                    let tclk = if self.clocks.hclk().0 == self.clocks.pclk().0 {
                        self.clocks.pclk().0
                    } else {
                        self.clocks.pclk().0 * 2
                    };
                    let ticks = tclk / frequency;

                    let psc = cast::u16((ticks - 1) / (1 << 16)).unwrap();
                    self.tim.psc.write(|w| w.psc().bits(psc));

                    let arr = cast::u16(ticks / cast::u32(psc + 1)).unwrap();
                    self.tim.arr.write(|w| unsafe { w.bits(cast::u32(arr)) });

                    // start counter
                    self.tim.cr1.modify(|_, w| w.cen().set_bit());
                }

                /// Return `Ok` if the timer has wrapped
                /// Automatically clears the flag and restarts the time
                fn wait(&mut self) -> nb::Result<(), Void> {
                    if self.tim.sr.read().uif().bit_is_clear() {
                        Err(nb::Error::WouldBlock)
                    } else {
                        self.tim.sr.modify(|_, w| w.uif().clear_bit());
                        Ok(())
                    }
                }
            }

            impl Periodic for Timer<$TIM> {}
        )+
    }
}

timers! {
    TIM1: (tim1, tim1en, tim1rst, apb2enr, apb2rstr),
    TIM3: (tim3, tim3en, tim3rst, apb1enr, apb1rstr),
    TIM14: (tim14, tim14en, tim14rst, apb1enr, apb1rstr),
    TIM16: (tim16, tim16en, tim16rst, apb2enr, apb2rstr),
    TIM17: (tim17, tim17en, tim17rst, apb2enr, apb2rstr),
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
    feature = "stm32f098",
))]
timers! {
    TIM2: (tim2, tim2en, tim2rst, apb1enr, apb1rstr),
}

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
timers! {
    TIM6: (tim6, tim6en, tim6rst, apb1enr, apb1rstr),
    TIM15: (tim15, tim15en, tim15rst, apb2enr, apb2rstr),
}

#[cfg(any(
    feature = "stm32f030xc",
    feature = "stm32f070xb",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
timers! {
    TIM7: (tim7, tim7en, tim7rst, apb1enr, apb1rstr),
}

use crate::gpio::{AF0, AF1, AF2, AF4, AF5};

use crate::gpio::{gpioa::*, gpiob::*, Alternate};

// Output channels marker traits
pub trait PinC1<TIM> {}
pub trait PinC1N<TIM> {}
pub trait PinC2<TIM> {}
pub trait PinC2N<TIM> {}
pub trait PinC3<TIM> {}
pub trait PinC3N<TIM> {}
pub trait PinC4<TIM> {}

macro_rules! channel_impl {
    ( $( $TIM:ident, $PINC:ident, $PINX:ident, $MODE:ident<$AF:ident>; )+ ) => {
        $(
            impl $PINC<$TIM> for $PINX<$MODE<$AF>> {}
        )+
    };
}

channel_impl!(
    TIM1, PinC1, PA8, Alternate<AF2>;
    TIM1, PinC1N, PA7, Alternate<AF2>;
    TIM1, PinC1N, PB13, Alternate<AF2>;
    TIM1, PinC2, PA9, Alternate<AF2>;
    TIM1, PinC2N, PB0, Alternate<AF2>;
    TIM1, PinC2N, PB14, Alternate<AF2>;
    TIM1, PinC3, PA10, Alternate<AF2>;
    TIM1, PinC3N, PB1, Alternate<AF2>;
    TIM1, PinC3N, PB15, Alternate<AF2>;
    TIM1, PinC4, PA11, Alternate<AF2>;

    TIM3, PinC1, PA6, Alternate<AF1>;
    TIM3, PinC2, PA7, Alternate<AF1>;

    TIM3, PinC1, PB4, Alternate<AF1>;
    TIM3, PinC2, PB5, Alternate<AF1>;
    TIM3, PinC3, PB0, Alternate<AF1>;
    TIM3, PinC4, PB1, Alternate<AF1>;


    TIM14, PinC1, PA4, Alternate<AF4>;
    TIM14, PinC1, PA7, Alternate<AF4>;
    TIM14, PinC1, PB1, Alternate<AF0>;

    TIM16, PinC1, PA6, Alternate<AF5>;
    TIM16, PinC1, PB8, Alternate<AF2>;
    TIM16, PinC1N, PB6, Alternate<AF2>;

    TIM17, PinC1, PA7, Alternate<AF5>;
    TIM17, PinC1, PB9, Alternate<AF2>;
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
channel_impl!(
    TIM15, PinC1, PA2, Alternate<AF0>;
    TIM15, PinC2, PA3, Alternate<AF0>;

    TIM15, PinC1, PB14, Alternate<AF1>;
    TIM15, PinC2, PB15, Alternate<AF1>;
);

#[cfg(any(
    feature = "stm32f030",
    feature = "stm32f051",
    feature = "stm32f058",
    feature = "stm32f070",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098"
))]
use crate::gpio::gpioc::*;

#[cfg(any(
    feature = "stm32f030",
    feature = "stm32f051",
    feature = "stm32f058",
    feature = "stm32f070",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098"
))]
channel_impl!(
    TIM3, PinC1, PC6, Alternate<AF0>;
    TIM3, PinC2, PC7, Alternate<AF0>;
    TIM3, PinC3, PC8, Alternate<AF0>;
    TIM3, PinC4, PC9, Alternate<AF0>;
);

#[cfg(any(
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098"
))]
use crate::gpio::gpioe::*;

#[cfg(any(
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098"
))]
channel_impl!(
    TIM1, PinC1, PE9, Alternate<AF0>;
    TIM1, PinC2, PE11, Alternate<AF0>;
    TIM1, PinC3, PE13, Alternate<AF0>;
    TIM1, PinC4, PE14, Alternate<AF0>;

    TIM3, PinC1, PE3, Alternate<AF0>;
    TIM3, PinC2, PE4, Alternate<AF0>;
    TIM3, PinC3, PE5, Alternate<AF0>;
    TIM3, PinC4, PE6, Alternate<AF0>;

    TIM16, PinC1, PE0, Alternate<AF0>;

    TIM17, PinC1, PE1, Alternate<AF0>;
);

#[cfg(any(
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
use crate::gpio::gpiof::*;

#[cfg(any(
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
channel_impl!(
    TIM15, PinC1, PF9, Alternate<AF0>;
    TIM15, PinC2, PF10, Alternate<AF0>;
);
