stm32f0xx-hal
=============
[![Travis](https://img.shields.io/travis/stm32-rs/stm32f0xx-hal.svg)](https://travis-ci.org/stm32-rs/stm32f0xx-hal)
[![Crates.io](https://img.shields.io/crates/v/stm32f0xx-hal.svg)](https://crates.io/crates/stm32f0xx-hal)
[![docs.rs](https://docs.rs/stm32f0xx-hal/badge.svg)](https://docs.rs/stm32f0xx-hal/)

_stm32f0xx-hal_ contains a hardware abstraction on top of the peripheral access
API for the STMicro STM32F0xx family of microcontrollers. It replaces the
[stm32f042-hal][] by a more ubiqitous version suitable for additional families.

Currently supported configuration are:
* stm32f030
* stm32f030x4
* stm32f030x6
* stm32f030x8
* stm32f030xc
* stm32f042
* stm32f070
* stm32f070x6
* stm32f070xb

The idea behind this crate is to gloss over the slight differences in the
various peripherals available on those MCUs so a HAL can be written for all
chips in that same family without having to cut and paste crates for every
single model.

Collaboration on this crate is highly welcome as are pull requests!

This crate relies on Adam Greigs fantastic [stm32f0][] crate to provide
appropriate register definitions and implements a partial set of the
[embedded-hal][] traits.

Some of the implementation was shamelessly adapted from the [stm32f103xx-hal][]
crate by Jorge Aparicio.

[stm32f0]: https://crates.io/crates/stm32f0
[stm32f042-hal]: https://github.com/therealprof/stm32f042-hal
[stm32f103xx-hal]: https://github.com/japaric/stm32f103xx-hal
[embedded-hal]: https://github.com/japaric/embedded-hal.git

License
-------

[0-clause BSD license](LICENSE-0BSD.txt).
