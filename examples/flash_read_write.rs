#![no_main]
#![no_std]

use panic_halt as _;
use stm32f0xx_hal as hal;

use crate::hal::{
    flash::{FlashExt, LockedFlash},
    pac,
    prelude::*,
};

use cortex_m_rt::entry;
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};

/// # NOTE
/// This example assumes a flash size of more than 16K. If your MCU has less or equal than 16K Bytes
/// of flash memory, adjust the `memory.x` file and `OFFSET_START` + `OFFSET_END` constants accordingly.
#[entry]
fn main() -> ! {
    if let Some(mut p) = pac::Peripherals::take() {
        let _ = p.RCC.configure().freeze(&mut p.FLASH);

        // Check that flash is big enough for this example
        assert!(p.FLASH.len() > 16 * 1024);

        // All examples use the first 16K of flash for the program so we use the first page after that
        const OFFSET_START: u32 = 16 * 1024;
        const OFFSET_END: u32 = OFFSET_START + 1024;
        // Unlock flash before writing
        let mut unlocked_flash = p.FLASH.unlocked();

        NorFlash::erase(&mut unlocked_flash, OFFSET_START, OFFSET_END).unwrap();

        // Write some data to the start of that page
        let write_data = [0xC0_u8, 0xFF_u8, 0xEE_u8, 0x00_u8];
        match NorFlash::write(&mut unlocked_flash, OFFSET_START, &write_data) {
            Err(_) => panic!(),
            Ok(_) => (),
        }

        // Read back the written data from flash
        let mut read_buffer: [u8; 4] = [0; 4];
        unlocked_flash.read(OFFSET_START, &mut read_buffer).unwrap();
        assert_eq!(write_data, read_buffer);

        // Lock flash by dropping it
        drop(unlocked_flash);

        // It is also possible to read "manually" using core functions
        let read_data = unsafe {
            core::slice::from_raw_parts(
                (p.FLASH.address() + OFFSET_START as usize) as *const u8,
                write_data.len(),
            )
        };
        for (i, d) in read_data.iter().enumerate() {
            read_buffer[i] = *d;
        }

        assert_eq!(write_data, read_buffer);

        // Reading is also possible on locked flash
        let mut locked_flash = LockedFlash::new(p.FLASH);
        locked_flash.read(OFFSET_START, &mut read_buffer).unwrap();

        assert_eq!(write_data, read_buffer);
    }
    loop {
        continue;
    }
}
