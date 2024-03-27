//! Device electronic signature
//!
//! (stored in flash memory)

/// This is the test voltage in millivolts of the calibration done at the factory
pub const VDDA_CALIB: u32 = 3300;

macro_rules! define_ptr_type {
    ($name: ident, $ptr: expr) => {
        impl $name {
            fn ptr() -> *const Self {
                $ptr as *const _
            }

            /// Returns a wrapped reference to the value in flash memory
            pub fn get() -> &'static Self {
                unsafe { &*Self::ptr() }
            }
        }
    };
}

// f030 and f070 don't have a UID in ROM
#[cfg(not(any(feature = "stm32f030", feature = "stm32f070")))]
#[derive(Hash, Debug)]
#[repr(C)]
pub struct Uid {
    x: u16,
    y: u16,
    waf_lot: [u8; 8],
}
#[cfg(not(any(feature = "stm32f030", feature = "stm32f070")))]
define_ptr_type!(Uid, 0x1FFF_F7AC);

/// Device UID from ROM. See the [reference manual](https://www.st.com/content/ccc/resource/technical/document/reference_manual/c2/f8/8a/f2/18/e6/43/96/DM00031936.pdf/files/DM00031936.pdf/jcr:content/translations/en.DM00031936.pdf#%5B%7B%22num%22%3A1575%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C67%2C755%2Cnull%5D) for more info.
#[cfg(not(any(feature = "stm32f030", feature = "stm32f070")))]
impl Uid {
    /// X coordinate on wafer
    pub fn x(&self) -> u16 {
        self.x
    }

    /// Y coordinate on wafer
    pub fn y(&self) -> u16 {
        self.y
    }

    /// Wafer number
    pub fn waf_num(&self) -> u8 {
        self.waf_lot[0]
    }

    /// Lot number
    pub fn lot_num(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.waf_lot[1..]) }
    }
}

/// Size of integrated flash
#[derive(Debug)]
#[repr(C)]
pub struct FlashSize(u16);
#[cfg(not(any(feature = "stm32f030", feature = "stm32f070")))]
define_ptr_type!(FlashSize, 0x1FFF_F7CC);
#[cfg(any(feature = "stm32f030", feature = "stm32f070"))]
define_ptr_type!(FlashSize, 0x1FFF_0000);

impl FlashSize {
    /// Read flash size in kilobytes
    pub fn kilo_bytes(&self) -> u16 {
        self.0
    }

    /// Read flash size in bytes
    pub fn bytes(&self) -> usize {
        usize::from(self.kilo_bytes()) * 1024
    }
}
