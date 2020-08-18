//! CDC-ACM serial port example using polling in a busy loop.
//! Target board: NUCLEO-F042K6
#![no_std]
#![no_main]

extern crate panic_halt;

use cortex_m_rt::entry;
use stm32f0xx_hal::usb::{Peripheral, UsbBus};
use stm32f0xx_hal::{pac, prelude::*};
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[entry]
fn main() -> ! {
    let mut dp = pac::Peripherals::take().unwrap();

    /* IMPORTANT: if you have a chip in TSSOP20 (STM32F042F) or UFQFPN28 (STM32F042G) package,
     * make sure you enable "usb-remap" feature on your Cargo.toml project, otherwise the device
     * will not USB enumerate.

       This feature flag enables clock for SYSCFG and remaps USB pins to PA9 and PA10.
    */

    let mut rcc = dp
        .RCC
        .configure()
        .hsi48()
        .enable_crs(dp.CRS)
        .sysclk(48.mhz())
        .pclk(24.mhz())
        .freeze(&mut dp.FLASH);

    // Configure the on-board LED (LD3, green)
    let gpiob = dp.GPIOB.split(&mut rcc);
    let mut led = cortex_m::interrupt::free(|cs| gpiob.pb3.into_push_pull_output(cs));
    led.set_low(); // Turn off

    let gpioa = dp.GPIOA.split(&mut rcc);

    let usb = Peripheral {
        usb: dp.USB,
        pin_dm: gpioa.pa11,
        pin_dp: gpioa.pa12,
    };
    let usb_bus = UsbBus::new(usb);

    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(USB_CLASS_CDC)
        .build();

    loop {
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }

        let mut buf = [0u8; 64];

        match serial.read(&mut buf) {
            Ok(count) if count > 0 => {
                led.set_high(); // Turn on

                // Echo back in upper case
                for c in buf[0..count].iter_mut() {
                    if 0x61 <= *c && *c <= 0x7a {
                        *c &= !0x20;
                    }
                }

                let mut write_offset = 0;
                while write_offset < count {
                    match serial.write(&buf[write_offset..count]) {
                        Ok(len) if len > 0 => {
                            write_offset += len;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        led.set_low(); // Turn off
    }
}
