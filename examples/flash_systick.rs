#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use crate::hal::gpio::*;
use crate::hal::prelude::*;
use crate::hal::stm32;

use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::syst::SystClkSource::Core;
use cortex_m::peripheral::Peripherals;
use cortex_m_rt::{entry, exception};

use core::cell::RefCell;
use core::ops::DerefMut;

static GPIO: Mutex<RefCell<Option<gpioa::PA1<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        cortex_m::interrupt::free(move |cs| {
            let mut flash = p.FLASH;
            let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut flash);

            let gpioa = p.GPIOA.split(&mut rcc);

            let mut syst = cp.SYST;

            /* (Re-)configure PA1 as output */
            let led = gpioa.pa1.into_push_pull_output(cs);

            *GPIO.borrow(cs).borrow_mut() = Some(led);

            /* Initialise SysTick counter with a defined value */
            unsafe { syst.cvr.write(1) };

            /* Set source for SysTick counter, here full operating frequency (== 8MHz) */
            syst.set_clock_source(Core);

            /* Set reload value, i.e. timer delay 48 MHz/4 Mcounts == 12Hz or 83ms */
            syst.set_reload(4_000_000 - 1);

            /* Start counter */
            syst.enable_counter();

            /* Start interrupt generation */
            syst.enable_interrupt();
        });
    }

    loop {
        continue;
    }
}

/* Define an exception, i.e. function to call when exception occurs. Here if our SysTick timer
 * trips the flash function will be called and the specified stated passed in via argument */
//, flash, state: u8 = 1);
#[exception]
fn SysTick() -> ! {
    static mut state: u8 = 1;

    /* Enter critical section */
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut led) = *GPIO.borrow(cs).borrow_mut().deref_mut() {
            /* Check state variable, keep LED off most of the time and turn it on every 10th tick */
            if *state < 10 {
                /* If set turn off the LED */
                led.set_low();

                /* And now increment state variable */
                *state += 1;
            } else {
                /* If not set, turn on the LED */
                led.set_high();

                /* And set new state variable back to 0 */
                *state = 0;
            }
        }
    });
}
