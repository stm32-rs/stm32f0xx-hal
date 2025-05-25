#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_halt as _;

use stm32f0xx_hal as _;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    loop {}
}
