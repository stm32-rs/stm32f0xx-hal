//! USB peripheral
//!
//! Requires the `stm32-usbd` feature.
//!
//! See <https://github.com/stm32-rs/stm32f0xx-hal/tree/master/examples>
//! for usage examples.

use crate::pac::{RCC, SYSCFG, USB};
use stm32_usbd::UsbPeripheral;

use crate::gpio::gpioa::{PA11, PA12};
use crate::gpio::{Floating, Input};
pub use stm32_usbd::UsbBus;

/*  TSSOP20 (STM32F042F) or UFQFPN28 (STM32F042G) packages equire `remap: true` for USB to function.
 *  This remapping sets the clock for SYSCFG and remaps USB pins to PA9 and PA10.
*/

pub struct Peripheral {
    pub usb: USB,
    pub pin_dm: PA11<Input<Floating>>,
    pub pin_dp: PA12<Input<Floating>>,
}

unsafe impl Sync for Peripheral {}

unsafe impl UsbPeripheral for Peripheral {
    const REGISTERS: *const () = USB::ptr() as *const ();
    const DP_PULL_UP_FEATURE: bool = true;
    const EP_MEMORY: *const () = 0x4000_6000 as _;
    const EP_MEMORY_SIZE: usize = 1024;

    fn enable() {
        let rcc = unsafe { &*RCC::ptr() };

        cortex_m::interrupt::free(|_| {
            // Enable USB peripheral
            rcc.apb1enr.modify(|_, w| w.usben().set_bit());

            // Reset USB peripheral
            rcc.apb1rstr.modify(|_, w| w.usbrst().set_bit());
            rcc.apb1rstr.modify(|_, w| w.usbrst().clear_bit());
        });
    }

    fn startup_delay() {
        // There is a chip specific startup delay. For STM32F103xx it's 1µs and this should wait for
        // at least that long.
        cortex_m::asm::delay(72);
    }
}

pub fn remap_pins(rcc: &mut RCC, syscfg: &mut SYSCFG) {
    cortex_m::interrupt::free(|_| {
        // Remap PA11/PA12 pins to PA09/PA10 for USB on
        // TSSOP20 (STM32F042F) or UFQFPN28 (STM32F042G) packages
        rcc.apb2enr.modify(|_, w| w.syscfgen().set_bit());
        syscfg.cfgr1.modify(|_, w| w.pa11_pa12_rmp().remapped());
    });
}

pub type UsbBusType = UsbBus<Peripheral>;
