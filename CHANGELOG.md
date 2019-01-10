# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Added

- Support for STM32F091 - @jessebraham
- Support for HSE as a system clocksource (#25 - breaking change) - @zklapow

### Changed

- Optimize delay implemenation (#42) - @david-sawatzke

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
- Support for STM32F070
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

[Unreleased]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.11.1...HEAD
[v0.11.1]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.11.0...v0.11.1
[v0.11.0]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.10.1...v0.11.0
[v0.10.1]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.10.0...v0.10.1
[v0.10.0]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.9.0...v0.10.0
[v0.9.0]: https://github.com/stm32-rs/stm32f0xx-hal/compare/v0.8.0...v0.9.0
