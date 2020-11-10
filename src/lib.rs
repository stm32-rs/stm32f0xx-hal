#![no_std]
#![allow(non_camel_case_types)]

pub use stm32f0;

#[cfg(any(feature = "stm32f030", feature = "stm32f070"))]
pub use stm32f0::stm32f0x0 as pac;

#[cfg(any(
    feature = "stm32f031",
    feature = "stm32f051",
    feature = "stm32f071",
    feature = "stm32f091",
))]
pub use stm32f0::stm32f0x1 as pac;

#[cfg(any(feature = "stm32f042", feature = "stm32f072"))]
pub use stm32f0::stm32f0x2 as pac;

#[cfg(any(
    feature = "stm32f038",
    feature = "stm32f048",
    feature = "stm32f058",
    feature = "stm32f078",
    feature = "stm32f098",
))]
pub use stm32f0::stm32f0x8 as pac;

#[cfg(feature = "device-selected")]
pub mod adc;
#[cfg(any(
    feature = "stm32f051",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
pub mod dac;
#[cfg(feature = "device-selected")]
pub mod delay;
#[cfg(feature = "device-selected")]
pub mod gpio;
#[cfg(feature = "device-selected")]
pub mod i2c;
#[cfg(feature = "device-selected")]
pub mod i2c_slave;
#[cfg(feature = "device-selected")]
pub mod prelude;
#[cfg(feature = "device-selected")]
pub mod rcc;
#[cfg(feature = "device-selected")]
pub mod serial;
#[cfg(feature = "device-selected")]
pub mod spi;
#[cfg(feature = "device-selected")]
pub mod time;
#[cfg(feature = "device-selected")]
pub mod timers;
#[cfg(any(
    feature = "stm32f031",
    feature = "stm32f051",
    feature = "stm32f071",
    feature = "stm32f091",
    feature = "stm32f042",
    feature = "stm32f072",
    feature = "stm32f038",
    feature = "stm32f048",
    feature = "stm32f058",
    feature = "stm32f078",
    feature = "stm32f098",
))]
pub mod tsc;
#[cfg(all(
    feature = "stm32-usbd",
    any(
        feature = "stm32f042",
        feature = "stm32f048",
        feature = "stm32f072",
        feature = "stm32f078",
        feature = "stm32f070x6",
        feature = "stm32f070xb",
    )
))]
pub mod usb;
#[cfg(feature = "device-selected")]
pub mod watchdog;

// TODO other controllers have CAN
#[cfg(any(
    feature = "stm32f091",
))]
pub mod can;

#[cfg(feature = "device-selected")]
#[deprecated(since = "0.17.0", note = "please use `pac` instead")]
pub use pac as stm32;
