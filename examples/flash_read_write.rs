#![no_main]
#![no_std]

use panic_halt as _;
use stm32f0xx_hal as hal;

use crate::hal::{flash::FlashExt, pac, prelude::*};

use cortex_m_rt::entry;
use embedded_storage::nor_flash::NorFlash;

#[entry]
fn main() -> ! {
    if let Some(mut p) = pac::Peripherals::take() {
        let _ = p.RCC.configure().freeze(&mut p.FLASH);

        // All examples use the first 16K of flash for the program so we use the first page after that
        const OFFSET_START: u32 = 32 * 1024;
        const OFFSET_END: u32 = OFFSET_START + 1024;
        // Unlock flash before writing
        let mut unlocked_flash = p.FLASH.unlocked();

        NorFlash::erase(&mut unlocked_flash, OFFSET_START, OFFSET_END).unwrap();
        NorFlash::erase(&mut unlocked_flash, 0x10000, 0x10000 + 1024).unwrap();

        // Write some data to the start of that page
        let write_data = [0xC0_u8, 0xFF_u8, 0xEE_u8, 0x00_u8];
        match NorFlash::write(&mut unlocked_flash, OFFSET_START, &write_data) {
            Err(e) => {
                let err = e;
                loop {}
            }
            Ok(_) => (),
        }

        // Lock flash by dropping it
        drop(unlocked_flash);

        // Read back the slice from flash
        let read_data = unsafe {
            core::slice::from_raw_parts(
                (p.FLASH.address() + 16 * 1024) as *const u8,
                write_data.len(),
            )
        };

        assert_eq!(write_data, *read_data);
    }
    loop {
        continue;
    }
}
