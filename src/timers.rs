#[cfg(feature = "stm32f030")]
use crate::stm32::{RCC, TIM1, TIM14, TIM15, TIM16, TIM17, TIM3, TIM6, TIM7};
#[cfg(feature = "stm32f042")]
use crate::stm32::{RCC, TIM1, TIM14, TIM16, TIM17, TIM2, TIM3};
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;

use crate::rcc::Clocks;
use cast::{u16, u32};
use embedded_hal::timer::{CountDown, Periodic};
use nb;
use void::Void;

use crate::time::Hertz;

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
    pub fn syst<T>(mut syst: SYST, timeout: T, clocks: Clocks) -> Self
    where
        T: Into<Hertz>,
    {
        syst.set_clock_source(SystClkSource::Core);
        let mut timer = Timer { tim: syst, clocks };
        timer.start(timeout);
        timer
    }

    /// Starts listening for an `event`
    pub fn listen(&mut self, event: Event) {
        match event {
            Event::TimeOut => self.tim.enable_interrupt(),
        }
    }

    /// Stops listening for an `event`
    pub fn unlisten(&mut self, event: Event) {
        match event {
            Event::TimeOut => self.tim.disable_interrupt(),
        }
    }
}

impl CountDown for Timer<SYST> {
    type Time = Hertz;

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
            impl Timer<$TIM> {
                // XXX(why not name this `new`?) bummer: constructors need to have different names
                // even if the `$TIM` are non overlapping (compare to the `free` function below
                // which just works)
                /// Configures a TIM peripheral as a periodic count down timer
                pub fn $tim<T>(tim: $TIM, timeout: T, clocks: Clocks) -> Self
                where
                    T: Into<Hertz>,
                {
                    // NOTE(unsafe) This executes only during initialisation
                    let rcc = unsafe { &(*RCC::ptr()) };

                    // enable and reset peripheral to a clean slate state
                    rcc.$apbenr.modify(|_, w| w.$timXen().set_bit());
                    rcc.$apbrstr.modify(|_, w| w.$timXrst().set_bit());
                    rcc.$apbrstr.modify(|_, w| w.$timXrst().clear_bit());

                    let mut timer = Timer {
                        clocks,
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
                    use crate::stm32::RCC;
                    let rcc = unsafe { &(*RCC::ptr()) };
                    // Pause counter
                    self.tim.cr1.modify(|_, w| w.cen().clear_bit());
                    // Disable timer
                    rcc.$apbenr.modify(|_, w| w.$timXen().clear_bit());
                    self.tim
                }
            }

            impl CountDown for Timer<$TIM> {
                type Time = Hertz;

                fn start<T>(&mut self, timeout: T)
                where
                    T: Into<Hertz>,
                {
                    // pause
                    self.tim.cr1.modify(|_, w| w.cen().clear_bit());
                    // restart counter
                    self.tim.cnt.reset();

                    let frequency = timeout.into().0;
                    let ticks = self.clocks.pclk().0 / frequency;

                    let psc = u16((ticks - 1) / (1 << 16)).unwrap();
                    self.tim.psc.write(|w| unsafe { w.psc().bits(psc) });

                    let arr = u16(ticks / u32(psc + 1)).unwrap();
                    self.tim.arr.write(|w| unsafe { w.bits(u32(arr)) });

                    // start counter
                    self.tim.cr1.modify(|_, w| w.cen().set_bit());
                }

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

#[cfg(any(feature = "stm32f030", feature = "stm32f042",))]
timers! {
    TIM1: (tim1, tim1en, tim1rst, apb2enr, apb2rstr),
    TIM3: (tim3, tim3en, tim3rst, apb1enr, apb1rstr),
    TIM14: (tim14, tim14en, tim14rst, apb1enr, apb1rstr),
    TIM16: (tim16, tim16en, tim16rst, apb2enr, apb2rstr),
    TIM17: (tim17, tim17en, tim17rst, apb2enr, apb2rstr),
}

#[cfg(any(feature = "stm32f030x8", feature = "stm32f030xc"))]
timers! {
    TIM6: (tim6, tim6en, tim6rst, apb1enr, apb1rstr),
    TIM15: (tim15, tim15en, tim15rst, apb2enr, apb2rstr),
}

#[cfg(feature = "stm32f030xc")]
timers! {
    TIM7: (tim7, tim7en, tim7rst, apb1enr, apb1rstr),
}

#[cfg(feature = "stm32f042")]
timers! {
    TIM2: (tim2, tim2en, tim2rst, apb1enr, apb1rstr),
}
