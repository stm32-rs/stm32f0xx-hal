use bxcan::{FilterOwner, Instance, RegisterBlock};

use crate::gpio::gpioa::{PA11, PA12};
use crate::gpio::gpiob::{PB8, PB9};
use crate::gpio::{Alternate, AF4};
use crate::pac::CAN;
use crate::rcc::Rcc;

mod sealed {
    pub trait Sealed {}
}

use self::sealed::Sealed;

pub use bxcan;

pub trait RxPin: Sealed {}
pub trait TxPin: Sealed {}

macro_rules! can_pins {
    (
        rx => [$($rx:ty),+ $(,)*],
        tx => [$($tx:ty),+ $(,)*],
    ) => {
        $(
            impl Sealed for $rx {}
            impl RxPin for $rx {}
        )+
        $(
            impl Sealed for $tx {}
            impl TxPin for $tx {}
        )+
    };
}

#[cfg(any(feature = "stm32f042", feature = "stm32f072", feature = "stm32f091"))]
can_pins! {
    rx => [PA11<Alternate<AF4>>, PB8<Alternate<AF4>>],
    tx => [PA12<Alternate<AF4>>, PB9<Alternate<AF4>>],
}

#[cfg(any(feature = "stm32f072", feature = "stm32f091"))]
use crate::gpio::{
    gpiod::{PD0, PD1},
    AF0,
};

#[cfg(any(feature = "stm32f072", feature = "stm32f091"))]
can_pins! {
    rx => [PD0<Alternate<AF0>>],
    tx => [PD1<Alternate<AF0>>],
}

/// Resources used by the CAN peripheral.
pub struct CanInstance<T: TxPin, R: RxPin> {
    peripheral: CAN,
    tx: T,
    rx: R,
}

impl<T: TxPin, R: RxPin> CanInstance<T, R> {
    pub fn new(peripheral: CAN, tx: T, rx: R, rcc: &mut Rcc) -> Self {
        rcc.regs.apb1enr.modify(|_, w| w.canen().enabled());
        rcc.regs.apb1rstr.modify(|_, w| w.canrst().reset());
        rcc.regs.apb1rstr.modify(|_, w| w.canrst().clear_bit());

        Self { peripheral, tx, rx }
    }

    pub fn into_raw(self) -> (CAN, T, R) {
        (self.peripheral, self.tx, self.rx)
    }

    /// Returns a reference to the raw CAN peripheral.
    pub unsafe fn peripheral(&mut self) -> &mut CAN {
        &mut self.peripheral
    }
}

unsafe impl<T: TxPin, R: RxPin> Instance for CanInstance<T, R> {
    const REGISTERS: *mut RegisterBlock = CAN::ptr() as *mut _;
}

unsafe impl<T: TxPin, R: RxPin> FilterOwner for CanInstance<T, R> {
    const NUM_FILTER_BANKS: u8 = 14;
}
