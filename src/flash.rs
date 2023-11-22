use core::convert::TryInto;
use core::{ptr, slice};

use embedded_storage::nor_flash::{
    ErrorType, MultiwriteNorFlash, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};

use crate::pac::FLASH;
use crate::signature::FlashSize;

/// Flash erase/program error
#[derive(Debug, Clone, Copy)]
pub enum Error {
    Programming,
    WriteProtection,
}

impl Error {
    fn read(flash: &FLASH) -> Option<Self> {
        let sr = flash.sr.read();
        if sr.pgerr().bit() {
            Some(Error::Programming)
        } else if sr.wrprt().bit() {
            Some(Error::WriteProtection)
        } else {
            None
        }
    }
}

/// Flash methods implemented for `pac::FLASH`
#[allow(clippy::len_without_is_empty)]
pub trait FlashExt {
    /// Memory-mapped address
    fn address(&self) -> usize;
    /// Size in bytes
    fn len(&self) -> usize;
    /// Returns a read-only view of flash memory
    fn read(&self) -> &[u8] {
        let ptr = self.address() as *const _;
        unsafe { slice::from_raw_parts(ptr, self.len()) }
    }
    /// Unlock flash for erasing/programming until this method's
    /// result is dropped
    fn unlocked(&mut self) -> UnlockedFlash;
    /// Returns flash memory sector of a given offset. Returns none if offset is out of range.
    fn sector(&self, offset: usize) -> Option<FlashSector>;
}

impl FlashExt for FLASH {
    fn address(&self) -> usize {
        0x0800_0000
    }

    fn len(&self) -> usize {
        FlashSize::get().bytes()
    }

    fn unlocked(&mut self) -> UnlockedFlash {
        unlock(self);
        UnlockedFlash { flash: self }
    }

    fn sector(&self, offset: usize) -> Option<FlashSector> {
        flash_sectors(self.len()).find(|s| s.contains(offset))
    }
}

/// Read-only flash
///
/// # Examples
///
/// ```
/// use stm32f0xx_hal::pac::Peripherals;
/// use stm32f0xx_hal::flash::LockedFlash;
/// use embedded_storage::nor_flash::ReadNorFlash;
///
/// let dp = Peripherals::take().unwrap();
/// let mut flash = LockedFlash::new(dp.FLASH);
/// println!("Flash capacity: {}", ReadNorFlash::capacity(&flash));
///
/// let mut buf = [0u8; 64];
/// ReadNorFlash::read(&mut flash, 0x0, &mut buf).unwrap();
/// println!("First 64 bytes of flash memory: {:?}", buf);
/// ```
pub struct LockedFlash {
    flash: FLASH,
}

impl LockedFlash {
    pub fn new(flash: FLASH) -> Self {
        Self { flash }
    }
}

impl FlashExt for LockedFlash {
    fn address(&self) -> usize {
        self.flash.address()
    }

    fn len(&self) -> usize {
        self.flash.len()
    }

    fn unlocked(&mut self) -> UnlockedFlash {
        self.flash.unlocked()
    }

    fn sector(&self, offset: usize) -> Option<FlashSector> {
        self.flash.sector(offset)
    }
}

/// Result of `FlashExt::unlocked()`
///
/// # Examples
///
/// ```
/// use stm32f0xx_hal::pac::Peripherals;
/// use stm32f0xx_hal::flash::{FlashExt, LockedFlash, UnlockedFlash};
/// use embedded_storage::nor_flash::NorFlash;
///
/// let dp = Peripherals::take().unwrap();
/// let mut flash = LockedFlash::new(dp.FLASH);
///
/// // Unlock flash for writing
/// let mut unlocked_flash = flash.unlocked();
///
/// // Erase the second 128 KB sector.
/// NorFlash::erase(&mut unlocked_flash, 128 * 1024, 256 * 1024).unwrap();
///
/// // Write some data at the start of the second 128 KB sector.
/// let buf = [0u8; 64];
/// NorFlash::write(&mut unlocked_flash, 128 * 1024, &buf).unwrap();
///
/// // Lock flash by dropping
/// drop(unlocked_flash);
/// ```
pub struct UnlockedFlash<'a> {
    flash: &'a mut FLASH,
}

/// Automatically lock flash erase/program when leaving scope
impl Drop for UnlockedFlash<'_> {
    fn drop(&mut self) {
        lock(self.flash);
    }
}

impl UnlockedFlash<'_> {
    /// Erase a flash page at offset
    ///
    /// Refer to the reference manual to see which sector corresponds
    /// to which memory address.
    pub fn erase(&mut self, offset: u32) -> Result<(), Error> {
        // Write address into the AR register
        self.flash
            .ar
            .write(|w| w.far().bits(self.flash.address() as u32 + offset));
        #[rustfmt::skip]
        self.flash.cr.modify(|_, w|
            w
                // page erase
                .per().set_bit()
                // start
                .strt().set_bit()
        );
        self.wait_ready();
        // Clear PER bit after operation is finished
        self.flash.cr.modify(|_, w| w.per().clear_bit());
        self.ok()
    }

    /// Program bytes with offset into flash memory
    pub fn program<'a, I>(&mut self, mut offset: usize, mut bytes: I) -> Result<(), Error>
    where
        I: Iterator<Item = &'a u8>,
    {
        if self.flash.cr.read().lock().bit_is_set() {
            return Err(Error::Programming);
        }
        let ptr = self.flash.address() as *mut u8;
        let mut bytes_written = 1;
        while bytes_written > 0 {
            bytes_written = 0;
            let amount = 2 - (offset % 2);

            #[allow(unused_unsafe)]
            self.flash.cr.modify(|_, w| unsafe {
                // programming
                w.pg().set_bit()
            });
            for _ in 0..amount {
                match bytes.next() {
                    Some(byte) => {
                        unsafe {
                            ptr::write_volatile(ptr.add(offset), *byte);
                        }
                        offset += 1;
                        bytes_written += 1;
                    }
                    None => break,
                }
            }
            self.wait_ready();
            self.ok()?;
        }
        self.flash.cr.modify(|_, w| w.pg().clear_bit());

        Ok(())
    }

    fn ok(&self) -> Result<(), Error> {
        Error::read(self.flash).map(Err).unwrap_or(Ok(()))
    }

    fn wait_ready(&self) {
        while self.flash.sr.read().bsy().bit() {}
    }
}

const UNLOCK_KEY1: u32 = 0x45670123;
const UNLOCK_KEY2: u32 = 0xCDEF89AB;

#[allow(unused_unsafe)]
fn unlock(flash: &FLASH) {
    flash.keyr.write(|w| unsafe { w.fkeyr().bits(UNLOCK_KEY1) });
    flash.keyr.write(|w| unsafe { w.fkeyr().bits(UNLOCK_KEY2) });
    assert!(!flash.cr.read().lock().bit())
}

fn lock(flash: &FLASH) {
    flash.cr.modify(|_, w| w.lock().set_bit());
}

/// Flash memory sector
pub struct FlashSector {
    /// Sector number
    pub number: u8,
    /// Offset from base memory address
    pub offset: usize,
    /// Sector size in bytes
    pub size: usize,
}

impl FlashSector {
    /// Returns true if given offset belongs to this sector
    pub fn contains(&self, offset: usize) -> bool {
        self.offset <= offset && offset < self.offset + self.size
    }
}

/// Iterator of flash memory sectors in a single bank.
/// Yields a size sequence of [16, 16, 16, 64, 128, 128, ..]
pub struct FlashSectorIterator {
    index: u8,
    start_sector: u8,
    start_offset: usize,
    end_offset: usize,
}

impl FlashSectorIterator {
    fn new(start_sector: u8, start_offset: usize, end_offset: usize) -> Self {
        Self {
            index: 0,
            start_sector,
            start_offset,
            end_offset,
        }
    }
}

impl Iterator for FlashSectorIterator {
    type Item = FlashSector;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start_offset >= self.end_offset {
            None
        } else {
            // F03x, F04x and F05x sectors are 1K long
            #[cfg(any(
                feature = "stm32f030",
                feature = "stm32f031",
                feature = "stm32f038",
                feature = "stm32f042",
                feature = "stm32f048",
                feature = "stm32f051",
                feature = "stm32f058",
            ))]
            let size = 1024;

            // F07x and F09x sectors are 2K long
            #[cfg(any(
                feature = "stm32f070",
                feature = "stm32f071",
                feature = "stm32f072",
                feature = "stm32f078",
                feature = "stm32f091",
                feature = "stm32f098",
            ))]
            let size = 2048;

            let sector = FlashSector {
                number: self.start_sector + self.index,
                offset: self.start_offset,
                size,
            };

            self.index += 1;
            self.start_offset += size;

            Some(sector)
        }
    }
}

/// Returns iterator of flash memory sectors for single and dual bank flash.
/// Sectors are returned in continuous memory order, while sector numbers can have spaces between banks.
pub fn flash_sectors(flash_size: usize) -> impl Iterator<Item = FlashSector> {
    // Chain an empty iterator to match types
    FlashSectorIterator::new(0, 0, flash_size).chain(FlashSectorIterator::new(0, 0, 0))
}

impl NorFlashError for Error {
    fn kind(&self) -> NorFlashErrorKind {
        NorFlashErrorKind::Other
    }
}

impl ErrorType for LockedFlash {
    type Error = Error;
}

impl ErrorType for UnlockedFlash<'_> {
    type Error = Error;
}

impl ReadNorFlash for LockedFlash {
    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let offset = offset as usize;
        bytes.copy_from_slice(&self.flash.read()[offset..offset + bytes.len()]);
        Ok(())
    }

    fn capacity(&self) -> usize {
        self.flash.len()
    }
}

impl<'a> ReadNorFlash for UnlockedFlash<'a> {
    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let offset = offset as usize;
        bytes.copy_from_slice(&self.flash.read()[offset..offset + bytes.len()]);
        Ok(())
    }

    fn capacity(&self) -> usize {
        self.flash.len()
    }
}

impl<'a> NorFlash for UnlockedFlash<'a> {
    const WRITE_SIZE: usize = 1;

    // Use largest sector size of 128 KB. All smaller sectors will be erased together.
    const ERASE_SIZE: usize = 128 * 1024;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        let mut current = from as usize;

        for sector in flash_sectors(self.flash.len()) {
            if sector.contains(current) {
                UnlockedFlash::erase(self, current as u32)?;
                current += sector.size;
            }

            if current >= to as usize {
                break;
            }
        }

        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.program(offset as usize, bytes.iter())
    }
}

// STM32F4 supports multiple writes
impl<'a> MultiwriteNorFlash for UnlockedFlash<'a> {}
