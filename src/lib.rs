#![no_std]
#![allow(non_camel_case_types)]

use bare_metal;
use cast;
use cortex_m;

pub use stm32f0;

#[cfg(feature = "stm32f042")]
pub use stm32f0::stm32f0x2 as stm32;

#[cfg(feature = "stm32f030")]
pub use stm32f0::stm32f0x0 as stm32;

// Enable use of interrupt macro
#[cfg(feature = "rt")]
pub use stm32f0::interrupt;

pub mod delay;
pub mod gpio;
pub mod i2c;
pub mod prelude;
pub mod rcc;
pub mod serial;
pub mod spi;
pub mod time;
