use core::convert::TryInto;
use core::{mem, ptr, slice};

use embedded_storage::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};

use crate::pac::FLASH;
use crate::signature::FlashSize;

/// First address of the flash memory
pub const FLASH_START: usize = 0x0800_0000;

// F03x, F04x and F05x pages are 1K long
#[cfg(any(
    feature = "stm32f030",
    feature = "stm32f031",
    feature = "stm32f038",
    feature = "stm32f042",
    feature = "stm32f048",
    feature = "stm32f051",
    feature = "stm32f058",
))]
pub const PAGE_SIZE: u32 = 1024;
// F03x, F04x and F05x have 64 flash pages
#[cfg(any(
    feature = "stm32f030",
    feature = "stm32f031",
    feature = "stm32f038",
    feature = "stm32f042",
    feature = "stm32f048",
    feature = "stm32f051",
    feature = "stm32f058",
))]
pub const NUM_PAGES: u32 = 64;

// F07x and F09x pages are 2K long
#[cfg(any(
    feature = "stm32f070",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
pub const PAGE_SIZE: u32 = 2048;
// F07x and F09x have 128 flash pages
#[cfg(any(
    feature = "stm32f070",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
pub const NUM_PAGES: u32 = 128;

/// Flash erase/program error
#[derive(Debug, Clone, Copy)]
pub enum Error {
    Programming,
    WriteProtection,
    /// STM32F0 can only write Half Words (16 Bit) to flash. Can not write to addresses not aligned to that.
    Alignment,
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
    fn read_all(&self) -> &[u8] {
        let ptr = self.address() as *const _;
        unsafe { slice::from_raw_parts(ptr, self.len()) }
    }
    /// Unlock flash for erasing/programming until this method's
    /// result is dropped
    fn unlocked(&mut self) -> UnlockedFlash;
}

impl FlashExt for FLASH {
    fn address(&self) -> usize {
        FLASH_START
    }

    fn len(&self) -> usize {
        FlashSize::get().bytes()
    }

    fn unlocked(&mut self) -> UnlockedFlash {
        unlock(self);
        UnlockedFlash { flash: self }
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

pub trait WriteErase {
    /// Native type for the flash for writing with the correct alignment and size
    ///
    /// Can be `u8`, `u16`, `u32`, ... (`u16` for STM32F0xx devices)
    type NativeType;

    /// The smallest possible write, depends on the platform
    fn program_native(&mut self, offset: usize, data: &[Self::NativeType]) -> Result<(), Error>;

    /// Write a buffer of bytes to memory and use native writes internally.
    /// If it is not the same length as a set of native writes the write will be padded to fill the
    /// native write.
    fn program(&mut self, offset: usize, data: &[u8]) -> Result<(), Error>;
}

impl WriteErase for UnlockedFlash<'_> {
    type NativeType = u16;

    fn program_native(&mut self, address: usize, data: &[Self::NativeType]) -> Result<(), Error> {
        // Wait for ready bit
        self.wait_ready();

        let mut addr = address as *mut Self::NativeType;

        // Write the data to flash
        for &half_word in data {
            self.flash.cr.modify(|_, w| w.pg().set_bit());
            unsafe {
                ptr::write_volatile(addr, half_word);
                addr = addr.add(1);
            }
        }

        self.wait_ready();

        // Clear programming bit
        self.flash.cr.modify(|_, w| w.pg().clear_bit());

        self.ok()
    }

    fn program(&mut self, mut address: usize, data: &[u8]) -> Result<(), Error> {
        if address % mem::align_of::<Self::NativeType>() != 0 {
            return Err(Error::Alignment);
        }

        let mut chunks = data.chunks_exact(mem::size_of::<Self::NativeType>());

        for exact_chunk in &mut chunks {
            let native = &[Self::NativeType::from_ne_bytes(
                exact_chunk.try_into().unwrap(),
            )];
            self.program_native(address, native)?;
            address += mem::size_of::<Self::NativeType>();
        }

        let remainder = chunks.remainder();

        if !remainder.is_empty() {
            let mut data = Self::NativeType::MAX;

            for b in remainder.iter().rev() {
                data = (data << 8) | *b as Self::NativeType;
            }

            let native = &[data];
            self.program_native(address, native)?;
        }

        self.ok()
    }
}

impl UnlockedFlash<'_> {
    /// Erase a flash page at offset
    ///
    /// Refer to the reference manual to see which sector corresponds
    /// to which memory address.
    pub fn erase(&mut self, offset: u32) -> Result<(), Error> {
        // Wait for ready bit
        self.wait_ready();

        // Set the PER (page erase) bit in CR register
        self.flash.cr.modify(|_, w| w.per().set_bit());

        // Write address into the AR register
        self.flash
            .ar
            .write(|w| w.far().bits(self.flash.address() as u32 + offset));
        // Set the STRT (start) Bit in CR register
        self.flash.cr.modify(|_, w| w.strt().set_bit());

        // Wait for the operation to finish
        self.wait_ready();

        // Clear PER bit after operation is finished
        self.flash.cr.modify(|_, w| w.per().clear_bit());
        self.ok()
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
        bytes.copy_from_slice(&self.flash.read_all()[offset..offset + bytes.len()]);
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
        bytes.copy_from_slice(&self.flash.read_all()[offset..offset + bytes.len()]);
        Ok(())
    }

    fn capacity(&self) -> usize {
        self.flash.len()
    }
}

impl<'a> NorFlash for UnlockedFlash<'a> {
    const WRITE_SIZE: usize = 2;

    const ERASE_SIZE: usize = PAGE_SIZE as usize;

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
        self.program(self.flash.address() + offset as usize, bytes)
    }
}
