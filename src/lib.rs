#![no_std]
#![allow(non_camel_case_types)]

extern crate bare_metal;
extern crate cast;
extern crate cortex_m;
pub extern crate embedded_hal as hal;

pub extern crate void;
pub use void::Void;

#[macro_use(block)]
pub extern crate nb;
pub use nb::block;

pub extern crate stm32f0;

#[cfg(feature = "stm32f042")]
pub use stm32f0::stm32f0x2 as stm32;

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
