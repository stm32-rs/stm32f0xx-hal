#![no_std]
#![allow(non_camel_case_types)]

pub use stm32f0;

#[cfg(feature = "stm32f042")]
pub use stm32f0::stm32f0x2 as stm32;

#[cfg(any(feature = "stm32f030", feature = "stm32f070"))]
pub use stm32f0::stm32f0x0 as stm32;

#[cfg(not(any(
    feature = "stm32f030",
    feature = "stm32f042",
    feature = "stm32f070"
)))]
pub mod stm32 {}

pub mod delay;
pub mod gpio;
pub mod i2c;
pub mod prelude;
pub mod rcc;
pub mod serial;
pub mod spi;
pub mod time;
pub mod timers;
pub mod watchdog;
