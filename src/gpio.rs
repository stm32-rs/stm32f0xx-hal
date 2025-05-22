//! General Purpose Input / Output

use core::convert::Infallible;
use core::marker::PhantomData;

use crate::rcc::Rcc;

/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The parts to split the GPIO into
    type Parts;

    /// Splits the GPIO block into independent pins and registers
    fn split(self, rcc: &mut Rcc) -> Self::Parts;
}

trait GpioRegExt {
    fn is_low(&self, pos: u8) -> bool;
    fn is_set_low(&self, pos: u8) -> bool;
    fn set_high(&self, pos: u8);
    fn set_low(&self, pos: u8);
}

/// Alternate function 0
pub struct AF0;
/// Alternate function 1
pub struct AF1;
/// Alternate function 2
pub struct AF2;
/// Alternate function 3
pub struct AF3;
/// Alternate function 4
pub struct AF4;
/// Alternate function 5
pub struct AF5;
/// Alternate function 6
pub struct AF6;
/// Alternate function 7
pub struct AF7;

/// Alternate function mode (type state)
pub struct Alternate<AF> {
    _mode: PhantomData<AF>,
}

/// Input mode (type state)
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Floating input (type state)
pub struct Floating;

/// Pulled down input (type state)
pub struct PullDown;

/// Pulled up input (type state)
pub struct PullUp;

/// Open drain input or output (type state)
pub struct OpenDrain;

/// Analog mode (type state)
pub struct Analog;

/// Output mode (type state)
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Push pull output (type state)
pub struct PushPull;

use embedded_hal::digital::v2::{toggleable, InputPin, OutputPin, StatefulOutputPin};

/// Fully erased pin
pub struct Pin<MODE> {
    i: u8,
    port: *const dyn GpioRegExt,
    _mode: PhantomData<MODE>,
}

// NOTE(unsafe) The only write access is to BSRR, which is thread safe
unsafe impl<MODE> Sync for Pin<MODE> {}
// NOTE(unsafe) this only enables read access to the same pin from multiple
// threads
unsafe impl<MODE> Send for Pin<MODE> {}

impl<MODE> StatefulOutputPin for Pin<Output<MODE>> {
    #[inline(always)]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        self.is_set_low().map(|v| !v)
    }

    #[inline(always)]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(unsafe { (*self.port).is_set_low(self.i) })
    }
}

impl<MODE> OutputPin for Pin<Output<MODE>> {
    type Error = Infallible;

    #[inline(always)]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe { (*self.port).set_high(self.i) };
        Ok(())
    }

    #[inline(always)]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe { (*self.port).set_low(self.i) }
        Ok(())
    }
}

impl<MODE> toggleable::Default for Pin<Output<MODE>> {}

impl InputPin for Pin<Output<OpenDrain>> {
    type Error = Infallible;

    #[inline(always)]
    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }

    #[inline(always)]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(unsafe { (*self.port).is_low(self.i) })
    }
}

impl<MODE> InputPin for Pin<Input<MODE>> {
    type Error = Infallible;

    #[inline(always)]
    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }

    #[inline(always)]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(unsafe { (*self.port).is_low(self.i) })
    }
}

impl<MODE> embedded_hal_1::digital::ErrorType for Pin<MODE> {
    type Error = Infallible;
}

impl<MODE> embedded_hal_1::digital::InputPin for Pin<MODE>
where
    Pin<MODE>: InputPin<Error = Infallible>,
{
    #[inline(always)]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        InputPin::is_high(self)
    }

    #[inline(always)]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        InputPin::is_low(self)
    }
}

impl<MODE> embedded_hal_1::digital::OutputPin for Pin<MODE>
where
    Pin<MODE>: OutputPin<Error = Infallible>,
{
    #[inline(always)]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        OutputPin::set_high(self)
    }

    #[inline(always)]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        OutputPin::set_low(self)
    }
}

impl<MODE> embedded_hal_1::digital::StatefulOutputPin for Pin<MODE>
where
    Pin<MODE>: StatefulOutputPin<Error = Infallible>,
{
    #[inline(always)]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        StatefulOutputPin::is_set_high(self)
    }

    #[inline(always)]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        StatefulOutputPin::is_set_low(self)
    }
}

macro_rules! gpio_trait {
    ($gpiox:ident) => {
        impl GpioRegExt for crate::pac::$gpiox::RegisterBlock {
            fn is_low(&self, pos: u8) -> bool {
                // NOTE(unsafe) atomic read with no side effects
                self.idr.read().bits() & (1 << pos) == 0
            }

            fn is_set_low(&self, pos: u8) -> bool {
                // NOTE(unsafe) atomic read with no side effects
                self.odr.read().bits() & (1 << pos) == 0
            }

            fn set_high(&self, pos: u8) {
                // NOTE(unsafe) atomic write to a stateless register
                unsafe { self.bsrr.write(|w| w.bits(1 << pos)) }
            }

            fn set_low(&self, pos: u8) {
                // NOTE(unsafe) atomic write to a stateless register
                unsafe { self.bsrr.write(|w| w.bits(1 << (pos + 16))) }
            }
        }
    };
}

gpio_trait!(gpioa);
gpio_trait!(gpiof);

macro_rules! gpio {
    ([$($GPIOX:ident, $gpiox:ident, $iopxenr:ident, $PXx:ident, $gate:meta => [
        $($PXi:ident: ($pxi:ident, $i:expr, $MODE:ty),)+
    ]),+]) => {
        $(
            /// GPIO
             #[cfg($gate)]
            pub mod $gpiox {
                use core::marker::PhantomData;
                use core::convert::Infallible;

                use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin, toggleable};
                use crate::{
                    rcc::Rcc,
                    pac::$GPIOX
                };

                use cortex_m::interrupt::CriticalSection;

                use super::{
                    Alternate, Analog, Floating, GpioExt, Input, OpenDrain, Output,
                    PullDown, PullUp, PushPull, AF0, AF1, AF2, AF3, AF4, AF5, AF6, AF7,
                    Pin, GpioRegExt,
                };

                /// GPIO parts
                pub struct Parts {
                    $(
                        /// Pin
                        pub $pxi: $PXi<$MODE>,
                    )+
                }

                impl GpioExt for $GPIOX {
                    type Parts = Parts;

                    fn split(self, rcc: &mut Rcc) -> Parts {
                        rcc.regs.ahbenr.modify(|_, w| w.$iopxenr().set_bit());

                        Parts {
                            $(
                                $pxi: $PXi { _mode: PhantomData },
                            )+
                        }
                    }
                }

                fn _set_alternate_mode (index:usize, mode: u32)
                {
                    let offset = 2 * index;
                    let offset2 = 4 * index;
                    unsafe {
                        let reg = &(*$GPIOX::ptr());
                        if offset2 < 32 {
                            reg.afrl.modify(|r, w| {
                                w.bits((r.bits() & !(0b1111 << offset2)) | (mode << offset2))
                            });
                        } else {
                            let offset2 = offset2 - 32;
                            reg.afrh.modify(|r, w| {
                                w.bits((r.bits() & !(0b1111 << offset2)) | (mode << offset2))
                            });
                        }
                        reg.moder.modify(|r, w| {
                            w.bits((r.bits() & !(0b11 << offset)) | (0b10 << offset))
                        });
                    }
                }

                $(
                    /// Pin
                    pub struct $PXi<MODE> {
                        _mode: PhantomData<MODE>,
                    }

                    impl<MODE> $PXi<MODE> {
                        /// Configures the pin to operate in AF0 mode
                        pub fn into_alternate_af0(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternate<AF0>> {
                            _set_alternate_mode($i, 0);
                            $PXi { _mode: PhantomData }
                        }

                        /// Configures the pin to operate in AF1 mode
                        pub fn into_alternate_af1(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternate<AF1>> {
                            _set_alternate_mode($i, 1);
                            $PXi { _mode: PhantomData }
                        }

                        /// Configures the pin to operate in AF2 mode
                        pub fn into_alternate_af2(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternate<AF2>> {
                            _set_alternate_mode($i, 2);
                            $PXi { _mode: PhantomData }
                        }

                        /// Configures the pin to operate in AF3 mode
                        pub fn into_alternate_af3(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternate<AF3>> {
                            _set_alternate_mode($i, 3);
                            $PXi { _mode: PhantomData }
                        }

                        /// Configures the pin to operate in AF4 mode
                        pub fn into_alternate_af4(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternate<AF4>> {
                            _set_alternate_mode($i, 4);
                            $PXi { _mode: PhantomData }
                        }

                        /// Configures the pin to operate in AF5 mode
                        pub fn into_alternate_af5(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternate<AF5>> {
                            _set_alternate_mode($i, 5);
                            $PXi { _mode: PhantomData }
                        }

                        /// Configures the pin to operate in AF6 mode
                        pub fn into_alternate_af6(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternate<AF6>> {
                            _set_alternate_mode($i, 6);
                            $PXi { _mode: PhantomData }
                        }

                        /// Configures the pin to operate in AF7 mode
                        pub fn into_alternate_af7(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternate<AF7>> {
                            _set_alternate_mode($i, 7);
                            $PXi { _mode: PhantomData }
                        }

                        /// Configures the pin to operate as a floating input pin
                        pub fn into_floating_input(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Input<Floating>> {
                            let offset = 2 * $i;
                            unsafe {
                                let reg = &(*$GPIOX::ptr());
                                reg.pupdr.modify(|r, w| {
                                    w.bits((r.bits() & !(0b11 << offset)) | (0b00 << offset))
                                });
                                reg.moder.modify(|r, w| {
                                    w.bits((r.bits() & !(0b11 << offset)) | (0b00 << offset))
                                });
                            }
                            $PXi { _mode: PhantomData }
                        }

                        /// Configures the pin to operate as a pulled down input pin
                        pub fn into_pull_down_input(
                            self, _cs: &CriticalSection
                            ) -> $PXi<Input<PullDown>> {
                            let offset = 2 * $i;
                            unsafe {
                                let reg = &(*$GPIOX::ptr());
                                reg.pupdr.modify(|r, w| {
                                    w.bits((r.bits() & !(0b11 << offset)) | (0b10 << offset))
                                });
                                reg.moder.modify(|r, w| {
                                    w.bits((r.bits() & !(0b11 << offset)) | (0b00 << offset))
                                });
                            }
                            $PXi { _mode: PhantomData }
                        }

                        /// Configures the pin to operate as a pulled up input pin
                        pub fn into_pull_up_input(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Input<PullUp>> {
                            let offset = 2 * $i;
                            unsafe {
                                let reg = &(*$GPIOX::ptr());
                                reg.pupdr.modify(|r, w| {
                                    w.bits((r.bits() & !(0b11 << offset)) | (0b01 << offset))
                                });
                                reg.moder.modify(|r, w| {
                                    w.bits((r.bits() & !(0b11 << offset)) | (0b00 << offset))
                                });
                            }
                            $PXi { _mode: PhantomData }
                        }

                        /// Configures the pin to operate as an analog pin
                        pub fn into_analog(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Analog> {
                            let offset = 2 * $i;
                            unsafe {
                                let reg = &(*$GPIOX::ptr());
                                reg.pupdr.modify(|r, w| {
                                    w.bits((r.bits() & !(0b11 << offset)) | (0b00 << offset))
                                });
                                reg.moder.modify(|r, w| {
                                    w.bits((r.bits() & !(0b11 << offset)) | (0b11 << offset))
                                });
                            }
                            $PXi { _mode: PhantomData }
                        }

                        /// Configures the pin to operate as an open drain output pin
                        pub fn into_open_drain_output(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Output<OpenDrain>> {
                            let offset = 2 * $i;
                            unsafe {
                                let reg = &(*$GPIOX::ptr());
                                reg.pupdr.modify(|r, w| {
                                    w.bits((r.bits() & !(0b11 << offset)) | (0b00 << offset))
                                });
                                reg.otyper.modify(|r, w| {
                                    w.bits(r.bits() | (0b1 << $i))
                                });
                                reg.moder.modify(|r, w| {
                                    w.bits((r.bits() & !(0b11 << offset)) | (0b01 << offset))
                                });
                            }
                            $PXi { _mode: PhantomData }
                        }

                        /// Configures the pin to operate as an push pull output pin
                        pub fn into_push_pull_output(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Output<PushPull>> {
                            let offset = 2 * $i;
                            unsafe {
                                let reg = &(*$GPIOX::ptr());
                                reg.pupdr.modify(|r, w| {
                                    w.bits((r.bits() & !(0b11 << offset)) | (0b00 << offset))
                                });
                                reg.otyper.modify(|r, w| {
                                    w.bits(r.bits() & !(0b1 << $i))
                                });
                                reg.moder.modify(|r, w| {
                                    w.bits((r.bits() & !(0b11 << offset)) | (0b01 << offset))
                                });
                            }
                            $PXi { _mode: PhantomData }
                        }

                        /// Configures the pin to operate as an push pull output pin with quick fall
                        /// and rise times
                        pub fn into_push_pull_output_hs(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Output<PushPull>> {
                            let offset = 2 * $i;
                            unsafe {
                                let reg = &(*$GPIOX::ptr());
                                reg.pupdr.modify(|r, w| {
                                    w.bits((r.bits() & !(0b11 << offset)) | (0b00 << offset))
                                });
                                reg.otyper.modify(|r, w| {
                                    w.bits(r.bits() & !(0b1 << $i))
                                });
                                reg.ospeedr.modify(|r, w| {
                                    w.bits(r.bits() & !(0b1 << $i))
                                });
                                reg.moder.modify(|r, w| {
                                    w.bits((r.bits() & !(0b11 << offset)) | (0b01 << offset))
                                });
                            }
                            $PXi { _mode: PhantomData }
                        }
                    }

                    impl $PXi<Output<OpenDrain>> {
                        /// Enables / disables the internal pull up
                        pub fn internal_pull_up(&mut self, _cs: &CriticalSection, on: bool) {
                            let offset = 2 * $i;
                            let value = if on { 0b01 } else { 0b00 };
                            unsafe {
                                let reg = &(*$GPIOX::ptr());
                                reg.pupdr.modify(|r, w| {
                                    w.bits((r.bits() & !(0b11 << offset)) | (value << offset))
                                });
                            }
                        }
                    }

                    impl<AF> $PXi<Alternate<AF>> {
                        /// Enables / disables the internal pull up
                        pub fn internal_pull_up(self, _cs: &CriticalSection, on: bool) -> Self {
                            let offset = 2 * $i;
                            let value = if on { 0b01 } else { 0b00 };
                            unsafe {
                                let reg = &(*$GPIOX::ptr());
                                reg.pupdr.modify(|r, w| {
                                    w.bits((r.bits() & !(0b11 << offset)) | (value << offset))
                                });
                            }
                            self
                        }
                    }

                    impl<AF> $PXi<Alternate<AF>> {
                        /// Turns pin alternate configuration pin into open drain
                        pub fn set_open_drain(self, _cs: &CriticalSection) -> Self {
                            let offset = $i;
                            unsafe {
                                let reg = &(*$GPIOX::ptr());
                                reg.otyper.modify(|r, w| {
                                    w.bits(r.bits() | (1 << offset))
                                });
                            }
                            self
                        }
                    }

                    impl<MODE> $PXi<Output<MODE>> {
                        /// Erases the pin number from the type
                        ///
                        /// This is useful when you want to collect the pins into an array where you
                        /// need all the elements to have the same type
                        pub fn downgrade(self) -> Pin<Output<MODE>> {
                            Pin {
                                i: $i,
                                port: $GPIOX::ptr() as *const dyn GpioRegExt,
                                _mode: self._mode,
                            }
                        }
                    }

                    impl<MODE> StatefulOutputPin for $PXi<Output<MODE>> {
                        fn is_set_high(&self) -> Result<bool, Self::Error> {
                            self.is_set_low().map(|v| !v)
                        }

                        fn is_set_low(&self) -> Result<bool, Self::Error> {
                            Ok(unsafe { (*$GPIOX::ptr()).is_set_low($i) })
                        }
                    }

                    impl<MODE> OutputPin for $PXi<Output<MODE>> {
                        type Error = Infallible;

                        fn set_high(&mut self) -> Result<(), Self::Error> {
                            Ok(unsafe { (*$GPIOX::ptr()).set_high($i) })
                        }

                        fn set_low(&mut self) -> Result<(), Self::Error> {
                            Ok(unsafe { (*$GPIOX::ptr()).set_low($i) })
                        }
                    }

                    impl<MODE> toggleable::Default for $PXi<Output<MODE>> {}

                    impl InputPin for $PXi<Output<OpenDrain>> {
                        type Error = Infallible;

                        fn is_high(&self) -> Result<bool, Self::Error> {
                            self.is_low().map(|v| !v)
                        }

                        fn is_low(&self) -> Result<bool, Self::Error> {
                            Ok(unsafe { (*$GPIOX::ptr()).is_low($i) })
                        }
                    }

                    impl<MODE> $PXi<Input<MODE>> {
                        /// Erases the pin number from the type
                        ///
                        /// This is useful when you want to collect the pins into an array where you
                        /// need all the elements to have the same type
                        pub fn downgrade(self) -> Pin<Input<MODE>> {
                            Pin {
                                i: $i,
                                port: $GPIOX::ptr() as *const dyn GpioRegExt,
                                _mode: self._mode,
                            }
                        }
                    }

                    impl<MODE> InputPin for $PXi<Input<MODE>> {
                        type Error = Infallible;

                        fn is_high(&self) -> Result<bool, Self::Error> {
                            self.is_low().map(|v| !v)
                        }

                        fn is_low(&self) -> Result<bool, Self::Error> {
                            Ok(unsafe { (*$GPIOX::ptr()).is_low($i) })
                        }
                    }

                    impl<MODE> embedded_hal_1::digital::ErrorType for $PXi<MODE>{
                        type Error = Infallible;
                    }

                    impl<MODE> embedded_hal_1::digital::InputPin for $PXi<MODE> where $PXi<MODE>: InputPin<Error=Infallible> {
                        #[inline(always)]
                        fn is_high(&mut self) -> Result<bool, Self::Error> {
                            InputPin::is_high(self)
                        }

                        #[inline(always)]
                        fn is_low(&mut self) -> Result<bool, Self::Error> {
                            InputPin::is_low(self)
                        }
                    }

                    impl<MODE> embedded_hal_1::digital::OutputPin for $PXi<MODE> where $PXi<MODE>: OutputPin<Error=Infallible> {
                        #[inline(always)]
                        fn set_high(&mut self) -> Result<(), Self::Error> {
                            OutputPin::set_high(self)
                        }

                        #[inline(always)]
                        fn set_low(&mut self) -> Result<(), Self::Error> {
                            OutputPin::set_low(self)
                        }
                    }

                    impl<MODE> embedded_hal_1::digital::StatefulOutputPin for $PXi<MODE> where $PXi<MODE>: StatefulOutputPin<Error=Infallible> {
                        #[inline(always)]
                        fn is_set_high(&mut self) -> Result<bool, Self::Error> {
                            StatefulOutputPin::is_set_high(self)
                        }

                        #[inline(always)]
                        fn is_set_low(&mut self) -> Result<bool, Self::Error> {
                            StatefulOutputPin::is_set_low(self)
                        }
                    }
                )+
            }
        )+
    }
}

gpio!([
    GPIOA, gpioa, iopaen, PA, any(
        feature = "device-selected"
    ) => [
        PA0: (pa0, 0, Input<Floating>),
        PA1: (pa1, 1, Input<Floating>),
        PA2: (pa2, 2, Input<Floating>),
        PA3: (pa3, 3, Input<Floating>),
        PA4: (pa4, 4, Input<Floating>),
        PA5: (pa5, 5, Input<Floating>),
        PA6: (pa6, 6, Input<Floating>),
        PA7: (pa7, 7, Input<Floating>),
        PA8: (pa8, 8, Input<Floating>),
        PA9: (pa9, 9, Input<Floating>),
        PA10: (pa10, 10, Input<Floating>),
        PA11: (pa11, 11, Input<Floating>),
        PA12: (pa12, 12, Input<Floating>),
        PA13: (pa13, 13, Input<Floating>),
        PA14: (pa14, 14, Input<Floating>),
        PA15: (pa15, 15, Input<Floating>),
    ],
    GPIOB, gpiob, iopben, PB, any(
        feature = "device-selected"
    ) => [
        PB0: (pb0, 0, Input<Floating>),
        PB1: (pb1, 1, Input<Floating>),
        PB2: (pb2, 2, Input<Floating>),
        PB3: (pb3, 3, Input<Floating>),
        PB4: (pb4, 4, Input<Floating>),
        PB5: (pb5, 5, Input<Floating>),
        PB6: (pb6, 6, Input<Floating>),
        PB7: (pb7, 7, Input<Floating>),
        PB8: (pb8, 8, Input<Floating>),
        PB9: (pb9, 9, Input<Floating>),
        PB10: (pb10, 10, Input<Floating>),
        PB11: (pb11, 11, Input<Floating>),
        PB12: (pb12, 12, Input<Floating>),
        PB13: (pb13, 13, Input<Floating>),
        PB14: (pb14, 14, Input<Floating>),
        PB15: (pb15, 15, Input<Floating>),
    ],
    GPIOC, gpioc, iopcen, PC, any(
        feature = "stm32f031",
        feature = "stm32f038",
        feature = "stm32f042",
        feature = "stm32f048"
    ) => [
        PC13: (pc13, 13, Input<Floating>),
        PC14: (pc14, 14, Input<Floating>),
        PC15: (pc15, 15, Input<Floating>),
    ],
    GPIOC, gpioc, iopcen, PC, any(
        feature = "stm32f030",
        feature = "stm32f051",
        feature = "stm32f058",
        feature = "stm32f070",
        feature = "stm32f071",
        feature = "stm32f072",
        feature = "stm32f078",
        feature = "stm32f091",
        feature = "stm32f098"
    ) => [
        PC0: (pc0, 0, Input<Floating>),
        PC1: (pc1, 1, Input<Floating>),
        PC2: (pc2, 2, Input<Floating>),
        PC3: (pc3, 3, Input<Floating>),
        PC4: (pc4, 4, Input<Floating>),
        PC5: (pc5, 5, Input<Floating>),
        PC6: (pc6, 6, Input<Floating>),
        PC7: (pc7, 7, Input<Floating>),
        PC8: (pc8, 8, Input<Floating>),
        PC9: (pc9, 9, Input<Floating>),
        PC10: (pc10, 10, Input<Floating>),
        PC11: (pc11, 11, Input<Floating>),
        PC12: (pc12, 12, Input<Floating>),
        PC13: (pc13, 13, Input<Floating>),
        PC14: (pc14, 14, Input<Floating>),
        PC15: (pc15, 15, Input<Floating>),
    ],
    GPIOD, gpiod, iopden, PD, any(
        feature = "stm32f030",
        feature = "stm32f051",
        feature = "stm32f058",
        feature = "stm32f070"
    ) => [
        PD2: (pd2, 2, Input<Floating>),
    ],
    GPIOD, gpiod, iopden, PD, any(
        feature = "stm32f071",
        feature = "stm32f072",
        feature = "stm32f078",
        feature = "stm32f091",
        feature = "stm32f098"
    ) => [
        PD0: (pd0, 0, Input<Floating>),
        PD1: (pd1, 1, Input<Floating>),
        PD2: (pd2, 2, Input<Floating>),
        PD3: (pd3, 3, Input<Floating>),
        PD4: (pd4, 4, Input<Floating>),
        PD5: (pd5, 5, Input<Floating>),
        PD6: (pd6, 6, Input<Floating>),
        PD7: (pd7, 7, Input<Floating>),
        PD8: (pd8, 8, Input<Floating>),
        PD9: (pd9, 9, Input<Floating>),
        PD10: (pd10, 10, Input<Floating>),
        PD11: (pd11, 11, Input<Floating>),
        PD12: (pd12, 12, Input<Floating>),
        PD13: (pd13, 13, Input<Floating>),
        PD14: (pd14, 14, Input<Floating>),
        PD15: (pd15, 15, Input<Floating>),
    ],
    GPIOE, gpioe, iopeen, PE, any(
        feature = "stm32f071",
        feature = "stm32f072",
        feature = "stm32f078",
        feature = "stm32f091",
        feature = "stm32f098"
    ) => [
        PE0: (pe0, 0, Input<Floating>),
        PE1: (pe1, 1, Input<Floating>),
        PE2: (pe2, 2, Input<Floating>),
        PE3: (pe3, 3, Input<Floating>),
        PE4: (pe4, 4, Input<Floating>),
        PE5: (pe5, 5, Input<Floating>),
        PE6: (pe6, 6, Input<Floating>),
        PE7: (pe7, 7, Input<Floating>),
        PE8: (pe8, 8, Input<Floating>),
        PE9: (pe9, 9, Input<Floating>),
        PE10: (pe10, 10, Input<Floating>),
        PE11: (pe11, 11, Input<Floating>),
        PE12: (pe12, 12, Input<Floating>),
        PE13: (pe13, 13, Input<Floating>),
        PE14: (pe14, 14, Input<Floating>),
        PE15: (pe15, 15, Input<Floating>),
    ],
    GPIOF, gpiof, iopfen, PF, any(
        feature = "stm32f030x4",
        feature = "stm32f030x6",
        feature = "stm32f030x8",
        feature = "stm32f051",
        feature = "stm32f058",
    ) => [
        PF0: (pf0, 0, Input<Floating>),
        PF1: (pf1, 1, Input<Floating>),
        PF4: (pf4, 4, Input<Floating>),
        PF5: (pf5, 5, Input<Floating>),
        PF6: (pf6, 6, Input<Floating>),
        PF7: (pf7, 7, Input<Floating>),
    ],
    GPIOF, gpiof, iopfen, PF, any(
        feature = "stm32f030xc",
        feature = "stm32f070"
    ) => [
        PF0: (pf0, 0, Input<Floating>),
        PF1: (pf1, 1, Input<Floating>),
    ],
    GPIOF, gpiof, iopfen, PF, any(
        feature = "stm32f031",
        feature = "stm32f038"
    ) => [
        PF0: (pf0, 0, Input<Floating>),
        PF1: (pf1, 1, Input<Floating>),
        PF6: (pf6, 6, Input<Floating>),
        PF7: (pf7, 7, Input<Floating>),
    ],
    GPIOF, gpiof, iopfen, PF, any(
        feature = "stm32f042",
        feature = "stm32f048"
    ) => [
        PF0: (pf0, 0, Input<Floating>),
        PF1: (pf1, 1, Input<Floating>),
        PF11: (pf11, 11, Input<Floating>),
    ],
    GPIOF, gpiof, iopfen, PF, any(
        feature = "stm32f071",
        feature = "stm32f072",
        feature = "stm32f078",
        feature = "stm32f091",
        feature = "stm32f098",
    ) => [
        PF0: (pf0, 0, Input<Floating>),
        PF1: (pf1, 1, Input<Floating>),
        PF2: (pf2, 2, Input<Floating>),
        PF3: (pf3, 3, Input<Floating>),
        PF6: (pf6, 6, Input<Floating>),
        PF9: (pf9, 9, Input<Floating>),
        PF10: (pf10, 10, Input<Floating>),
    ]
]);
