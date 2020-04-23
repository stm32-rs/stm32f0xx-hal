stm32f0xx-hal
=============

[![Travis](https://img.shields.io/travis/stm32-rs/stm32f0xx-hal.svg)](https://travis-ci.org/stm32-rs/stm32f0xx-hal)
[![Crates.io](https://img.shields.io/crates/v/stm32f0xx-hal.svg)](https://crates.io/crates/stm32f0xx-hal)
[![docs.rs](https://docs.rs/stm32f0xx-hal/badge.svg)](https://docs.rs/stm32f0xx-hal/)

[_stm32f0xx-hal_](https://github.com/stm32-rs/stm32f0xx-hal) contains a hardware abstraction on top of the peripheral access API for the STMicro STM32F0xx family of microcontrollers.

This crate replaces the [stm32f042-hal](https://github.com/therealprof/stm32f042-hal) by a more ubiquitous version suitable for additional families. The idea behind this crate is to gloss over the slight differences in the various peripherals available on those MCUs so a HAL can be written for all chips in that same family without having to cut and paste crates for every single model.

This crate relies on Adam Greig's fantastic [stm32f0](https://crates.io/crates/stm32f0) crate to provide appropriate register definitions, and implements a partial set of the [embedded-hal](https://github.com/rust-embedded/embedded-hal) traits. Some of the implementation was shamelessly adapted from the [stm32f103xx-hal](https://github.com/japaric/stm32f103xx-hal) crate by Jorge Aparicio.

Collaboration on this crate is highly welcome, as are pull requests!


Supported Configurations
------------------------

* __stm32f030__ (stm32f030x4, stm32f030x6, stm32f030x8, stm32f030xc)  
* __stm32f031__  
* __stm32f038__  
* __stm32f042__  
* __stm32f048__  
* __stm32f051__  
* __stm32f058__  
* __stm32f070__ (stm32f070x6, stm32f070xb)  
* __stm32f071__  
* __stm32f072__  
* __stm32f078__  
* __stm32f091__  
* __stm32f098__  


Getting Started
---------------
The `examples` folder contains several example programs. To compile them, one must specify the target device as cargo feature:
```
$ cargo build --features=stm32f030
```

To use stm32f0xx-hal as a dependency in a standalone project the target device feature must be specified in the `Cargo.toml` file:
```
[dependencies]
cortex-m = "0.6.0"
cortex-m-rt = "0.6.8"
stm32f0xx-hal = {version = "0.16", features = ["stm32f030"]}
```

If you are unfamiliar with embedded development using Rust, there are a number of fantastic resources available to help.

- [Embedded Rust Documentation](https://docs.rust-embedded.org/)  
- [The Embedded Rust Book](https://docs.rust-embedded.org/book/)  
- [Rust Embedded FAQ](https://docs.rust-embedded.org/faq.html)  
- [rust-embedded/awesome-embedded-rust](https://github.com/rust-embedded/awesome-embedded-rust)


Minimum supported Rust version
------------------------------

The minimum supported Rust version at the moment is **1.39.0**. Older versions
**may** compile, especially when some features are not used in your
application.

Changelog
---------

See [CHANGELOG.md](CHANGELOG.md).


License
-------

0-Clause BSD License, see [LICENSE-0BSD.txt](LICENSE-0BSD.txt) for more details.
