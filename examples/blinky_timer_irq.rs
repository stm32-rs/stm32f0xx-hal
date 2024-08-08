#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal as hal;

use crate::hal::{
    gpio::*,
    pac::{interrupt, Interrupt, Peripherals, TIM7},
    prelude::*,
    time::Hertz,
    timers::*,
};

use cortex_m_rt::entry;

use bare_metal::Mutex;
use core::cell::RefCell;
use cortex_m::peripheral::Peripherals as c_m_Peripherals;

// A type definition for the GPIO pin to be used for our LED
type LEDPIN = gpioa::PA5<Output<PushPull>>;

// Make LED pin globally available
static GLED: Mutex<RefCell<Option<LEDPIN>>> = Mutex::new(RefCell::new(None));

// Make timer interrupt registers globally available
static GINT: Mutex<RefCell<Option<Timer<TIM7>>>> = Mutex::new(RefCell::new(None));

// Define an interupt handler, i.e. function to call when interrupt occurs. Here if our external
// interrupt trips when the timer timed out
#[interrupt]
fn TIM7() {
    static mut LED: Option<LEDPIN> = None;
    static mut INT: Option<Timer<TIM7>> = None;

    let led = LED.get_or_insert_with(|| {
        cortex_m::interrupt::free(|_| {
            // SAFETY: We are in a critical section, but the `cortex_m` critical section
            // token is not compatible with the `bare_metal` token. Future version of the
            // `cortex_m` crate will not supply *any* token to this callback!
            let cs = unsafe { bare_metal::CriticalSection::new() };

            // Move LED pin here, leaving a None in its place
            GLED.borrow(cs).replace(None).unwrap()
        })
    });

    let int = INT.get_or_insert_with(|| {
        cortex_m::interrupt::free(|_| {
            // SAFETY: We are in a critical section, but the `cortex_m` critical section
            // token is not compatible with the `bare_metal` token. Future version of the
            // `cortex_m` crate will not supply *any* token to this callback!
            let cs = unsafe { bare_metal::CriticalSection::new() };

            // Move LED pin here, leaving a None in its place
            GINT.borrow(cs).replace(None).unwrap()
        })
    });

    led.toggle().ok();
    int.wait().ok();
}

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (Peripherals::take(), c_m_Peripherals::take()) {
        cortex_m::interrupt::free(move |_| {
            // SAFETY: We are in a critical section, but the `cortex_m` critical section
            // token is not compatible with the `bare_metal` token. Future version of the
            // `cortex_m` crate will not supply *any* token to this callback!
            let cs = unsafe { &bare_metal::CriticalSection::new() };

            let mut rcc = p
                .RCC
                .configure()
                .hsi48()
                .enable_crs(p.CRS)
                .sysclk(48.mhz())
                .pclk(24.mhz())
                .freeze(&mut p.FLASH);

            let gpioa = p.GPIOA.split(&mut rcc);

            // (Re-)configure PA5 as output
            let led = gpioa.pa5.into_push_pull_output(cs);

            // Move the pin into our global storage
            *GLED.borrow(*cs).borrow_mut() = Some(led);

            // Set up a timer expiring after 1s
            let mut timer = Timer::tim7(p.TIM7, Hertz(1), &mut rcc);

            // Generate an interrupt when the timer expires
            timer.listen(Event::TimeOut);

            // Move the timer into our global storage
            *GINT.borrow(*cs).borrow_mut() = Some(timer);

            // Enable TIM7 IRQ, set prio 1 and clear any pending IRQs
            let mut nvic = cp.NVIC;
            unsafe {
                nvic.set_priority(Interrupt::TIM7, 1);
                cortex_m::peripheral::NVIC::unmask(Interrupt::TIM7);
            }
            cortex_m::peripheral::NVIC::unpend(Interrupt::TIM7);
        });
    }

    loop {
        continue;
    }
}
