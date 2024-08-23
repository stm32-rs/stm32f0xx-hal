#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal as hal;

use crate::hal::{gpio::*, pac, prelude::*};

use bare_metal::Mutex;
use cortex_m::{peripheral::syst::SystClkSource::Core, Peripherals};
use cortex_m_rt::{entry, exception};

use core::cell::RefCell;
use core::mem::swap;

// A type definition for the GPIO pin to be used for our LED
type LEDPIN = gpiob::PB3<Output<PushPull>>;

// Mutex protected structure for our shared GPIO pin
static GPIO: Mutex<RefCell<Option<LEDPIN>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (pac::Peripherals::take(), Peripherals::take()) {
        cortex_m::interrupt::free(move |_| {
            // SAFETY: We are in a critical section, but the `cortex_m` critical section
            // token is not compatible with the `bare_metal` token. Future version of the
            // `cortex_m` crate will not supply *any* token to this callback!
            let cs = unsafe { &bare_metal::CriticalSection::new() };

            let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);

            // Get access to individual pins in the GPIO port
            let gpioa = p.GPIOB.split(&mut rcc);

            // (Re-)configure the pin connected to our LED as output
            let led = gpioa.pb3.into_push_pull_output(cs);

            // Transfer GPIO into a shared structure
            swap(&mut Some(led), &mut GPIO.borrow(*cs).borrow_mut());

            let mut syst = cp.SYST;

            // Initialise SysTick counter with a defined value
            unsafe { syst.cvr.write(1) };

            // Set source for SysTick counter, here full operating frequency (== 48MHz)
            syst.set_clock_source(Core);

            // Set reload value, i.e. timer delay 48 MHz/4 Mcounts == 12Hz or 83ms
            syst.set_reload(4_000_000 - 1);

            // Start counting
            syst.enable_counter();

            // Enable interrupt generation
            syst.enable_interrupt();
        });
    }

    loop {
        continue;
    }
}

// Define an exception handler, i.e. function to call when exception occurs. Here, if our SysTick
// timer generates an exception the following handler will be called
#[exception]
fn SysTick() {
    // Our moved LED pin
    static mut LED: Option<LEDPIN> = None;

    // Exception handler state variable
    static mut STATE: u8 = 1;

    // If LED pin was moved into the exception handler, just use it
    if let Some(led) = LED {
        // Check state variable, keep LED off most of the time and turn it on every 10th tick
        if *STATE < 10 {
            // Turn off the LED
            led.set_low().ok();

            // And now increment state variable
            *STATE += 1;
        } else {
            // Turn on the LED
            led.set_high().ok();

            // And set new state variable back to 0
            *STATE = 0;
        }
    }
    // Otherwise move it out of the Mutex protected shared region into our exception handler
    else {
        // Enter critical section
        cortex_m::interrupt::free(|_| {
            // SAFETY: We are in a critical section, but the `cortex_m` critical section
            // token is not compatible with the `bare_metal` token. Future version of the
            // `cortex_m` crate will not supply *any* token to this callback!
            let cs = unsafe { bare_metal::CriticalSection::new() };

            // Swap globally stored data with SysTick private data
            swap(LED, &mut GPIO.borrow(cs).borrow_mut());
        });
    }
}
