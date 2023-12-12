# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Changed

- Updated the `cast` dependency from 0.2 to 0.3

### Added

- Provide getters to serial status flags idle/txe/rxne/tc.
- Provide ability to reset timer UIF interrupt flag
- PWM complementary output capability for TIM1 with new example to demonstrate
- PWM output on complementary channels only for single channel timers (TIM16 + TIM17)

### Fixed

- Wrong mode when using PWM channel 2 of a two-channel timer
- `adc_values` example conversion error
- `invalid_reference_casting` Compilation error in spi.rs for Rust version 1.73+ (
  See [PR#112431](https://github.com/rust-lang/rust/pull/112431) for more info)
- `unused_doc_comments` Warning in rcc.rs

## [v0.18.0] - 2021-11-14

### Changed

- Updated stm32f0 dependency to v0.13 (breaking change)
- Move SPI busy polling to `check_send()`
- Poll for SPI transaction completion before returning
- Update `remap_pins()` and remove critical section
- Updated `stm32f0` peripheral access crate from 0.13 to 0.14
- Updated the `bxcan` dependency from 0.4.0 to 0.6.0
- Address a few clippy lints
- Fix `VTemp::convert_temp()`

### Added

- Add CAN bus abstraction based on the [bxcan] crate.
- Add PWM output generation based on the timers.
- Added `impl From<KiloHertz> for Hertz`
- Added `impl From<MegaHertz> for Hertz`
- Added `impl From<KiloHertz> for MegaHertz`
- Added `impl From<Hertz> for IwdgTimeout`

[bxcan]: https://crates.io/crates/bxcan

## [v0.17.1] - 2020-08-30

### Changed

- Simplify USB PA11/12 remapping for STM32F042x via `usb_bus.usb_remap()` function.

### Added

- Complete the `TscPin` trait implementation for all touch pins in the f0 family

## [v0.17.0] - 2020-06-27

### Changed

- Remove duplicate error bits clearing in serial `read()` implementation
- Optimize SPI implementation
- Use `pac` instead of `stm32` for PAC access and soft-deprecate the former
- Updated stm32f0 dependency to v0.11 (breaking change)

### Added

- Add 16bit SPI transfers
- Another example resembling a stop watch controlled via serial interface

### Fixed

- Incorrect PLLSRC bits when using HSE

## [v0.16.0] - 2020-02-02

### Added

- Another blinky example using a timer interrupt

### Changed

- Added "bypass" parameter to Rcc HSE configuration (breaking change)
- Add "usbsrc" function to Rcc configuration, used for selecting USB clock source
- For STM32F030, require use more specific feature flag, e.g. "stm32f030xc"
- Add `embedded-hal` `blocking::i2c::Read` implementation for I2C
- Added USB driver

### Fixed

- Timer: Fix use of wrong frequency when HCLK != PCLK
- RCC: Correct code to enable PLL
- RCC: Correct calculation of PLL multiplier

## [v0.15.2] - 2019-11-04

### Changed

- Re-enabled LTO
- Changed digital pin functionality to implement v2 versions
- Fixed a few deprecation warning and lints
- Enabled commented out and now available GPIOE support for 07x and 09x families
- Extract register block address only once
- Add DAC driver

## [v0.15.1] - 2019-08-11

### Fixed

- Clear UART errors in hardware after handling them

## [v0.15.0] - 2019-08-09

### Changed

- Updated stm32f0 dependency to v0.8.0 (breaking change)
- Made blinky example more universal by reducing CS

### Added

- Added fancier example moving a resource into an exception handler

## [v0.14.1] - 2019-06-06

### Added

- Support for CRS for devices with USB and HSI48

### Changed

- Clear error flags in serial read() before returning
- Revised feature flags for HSI48 clock support

## [v0.14.0] - 2019-04-25

### Changed

- Updated stm32f0 dependency to v0.7.0 (breaking change) - @jessebraham
- Bumped cortex-m dependency to ">=0.5.8,<0.7.0" to let user decide version
- Bumped cortex-m-rt dependency to v0.6.8

## [v0.13.0] - 2019-02-06

### Added

- Support for stm32f0x8 line - @jessebraham
- Support for capacitive touch sensing (TSC)

### Changed

- Updated to stm32-rs v0.6.0 - @HarkonenBade
- Updated the ADC code to use variants added in stm32-rs v0.6.0 - @HarkonenBade
- Improved serial `write_str` implementation

### Fixed

- Fixed ADC use trampling over the HSI48 clock settings

## [v0.12.0] - 2019-01-13

### Added

- Support for stm32f0x1 line - @jessebraham
- Support for HSE as a system clocksource (#25 - breaking change) - @zklapow
- Add ability to use a Tx/Rx only serial instance - @david-sawatzke

### Changed

- Optimize delay implemenation (#42) - @david-sawatzke
- Enforced more rigorous safety guarentees (#41 - Very breaking change) - @HarkonenBade

### Fixed

- Fixed panic in delay overflow handling for debug builds - @david-sawatzke

## [v0.11.1] - 2019-01-05

### Added

- Added peripheral definitions for the stm32f072xx line - @Yatekii

### Changed

- Fixed broken PC GPIO definitions with feature = "stm32f030" and feature = "stm32f070"
- More robust error handling for I2C

## [v0.11.0] - 2019-01-04

### Added

- Added ADC helper functions to read more intuitive values (#22) - @HarkonenBade
- Added interrupt enabling/disabling support to USART ports
- Added the option to have multiple Delay instances by cloning it - @david-sawatzke

### Changed

- Fixed a few clippy lints

### Removed

- Removed the free() method on the Delay provider (breaking change)

## [v0.10.1] - 2018-12-25

### Added

- Added Sync & Send ability to Pin
- Added initial implementation of an ADC interface (#13) - @HarkonenBade
- Added virtual-feature "device-selected" to simplify feature gating

### Changed

- Added overflow guards to delay

## [v0.10.0] - 2018-12-23

### Added

- Reworked GPIOs and added fully erased pins
- Timer support
- Support for STM32F070 - @jessebraham
- Additional peripheral support for STM32F030
- Watchdog support

### Changed

- Removed superfluous use statements
- Re-added Send ability for U(S)ART Rx/Tx
- Made crate to compile without features
- Eliminated a lot of unused warnings

### Fixed

- Fixed some comments
- Changed some prelude aliases to reflect crate name

### Removed

- Examples requiring additional driver crates

## [v0.9.0] - 2018-12-17

### Added

- Toggleable implementation for GPIOs
- Initial support for STM32F030
- LICENSE file

### Changed

- Updated stm32f0 dependency to v0.5.0.
- Interrupt handler to new #[interrupt] attribute

[Unreleased]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.18.0...HEAD
[v0.18.0]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.17.1...v0.18.0
[v0.17.1]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.17.0...v0.17.1
[v0.17.0]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.16.0...v0.17.0
[v0.16.0]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.15.2...v0.16.0
[v0.15.2]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.15.1...v0.15.2
[v0.15.1]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.15.0...v0.15.1
[v0.15.0]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.14.1...v0.15.0
[v0.14.1]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.14.0...v0.14.1
[v0.14.0]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.13.0...v0.14.0
[v0.13.0]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.12.0...v0.13.0
[v0.12.0]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.11.1...v0.12.0
[v0.11.1]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.11.0...v0.11.1
[v0.11.0]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.10.1...v0.11.0
[v0.10.1]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.10.0...v0.10.1
[v0.10.0]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.9.0...v0.10.0
[v0.9.0]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.8.0...v0.9.0
