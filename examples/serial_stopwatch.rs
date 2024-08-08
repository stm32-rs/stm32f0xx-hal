#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal as hal;

use crate::hal::{
    pac::{interrupt, Interrupt, Peripherals, TIM7},
    prelude::*,
    serial::Serial,
    timers::{Event, Timer},
};
use core::cell::RefCell;
use core::fmt::Write as _;
use core::ops::DerefMut;

use bare_metal::Mutex;
use cortex_m::peripheral::Peripherals as c_m_Peripherals;
use cortex_m_rt::entry;

// Make timer interrupt registers globally available
static GINT: Mutex<RefCell<Option<Timer<TIM7>>>> = Mutex::new(RefCell::new(None));

#[derive(Copy, Clone)]
struct Time {
    seconds: u32,
    millis: u16,
}

static TIME: Mutex<RefCell<Time>> = Mutex::new(RefCell::new(Time {
    seconds: 0,
    millis: 0,
}));

// Define an interupt handler, i.e. function to call when interrupt occurs. Here if our external
// interrupt trips when the timer timed out
#[interrupt]
fn TIM7() {
    cortex_m::interrupt::free(|_| {
        // SAFETY: We are in a critical section, but the `cortex_m` critical section
        // token is not compatible with the `bare_metal` token. Future version of the
        // `cortex_m` crate will not supply *any* token to this callback!
        let cs = unsafe { bare_metal::CriticalSection::new() };

        // Move LED pin here, leaving a None in its place
        GINT.borrow(cs)
            .borrow_mut()
            .deref_mut()
            .as_mut()
            .unwrap()
            .wait()
            .ok();
        let mut time = TIME.borrow(cs).borrow_mut();
        time.millis += 1;
        if time.millis == 1000 {
            time.millis = 0;
            time.seconds += 1;
        }
    });
}

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (Peripherals::take(), c_m_Peripherals::take()) {
        let mut serial = cortex_m::interrupt::free(move |_| {
            // SAFETY: We are in a critical section, but the `cortex_m` critical section
            // token is not compatible with the `bare_metal` token. Future version of the
            // `cortex_m` crate will not supply *any* token to this callback!
            let cs = unsafe { &bare_metal::CriticalSection::new() };

            let mut flash = p.FLASH;
            let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut flash);

            // Use USART2 with PA2 and PA3 as serial port
            let gpioa = p.GPIOA.split(&mut rcc);
            let tx = gpioa.pa2.into_alternate_af1(cs);
            let rx = gpioa.pa3.into_alternate_af1(cs);

            // Set up a timer expiring every millisecond
            let mut timer = Timer::tim7(p.TIM7, 1000.hz(), &mut rcc);

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

            // Set up our serial port
            Serial::usart2(p.USART2, (tx, rx), 115_200.bps(), &mut rcc)
        });

        // Print a welcome message
        writeln!(
            serial,
            "Welcome to the stop watch, hit any key to see the current value and 0 to reset\r",
        )
        .ok();

        loop {
            // Wait for reception of a single byte
            let received = nb::block!(serial.read()).unwrap();

            let time = cortex_m::interrupt::free(|_| {
                // SAFETY: We are in a critical section, but the `cortex_m` critical section
                // token is not compatible with the `bare_metal` token. Future version of the
                // `cortex_m` crate will not supply *any* token to this callback!
                let cs = unsafe { bare_metal::CriticalSection::new() };

                let mut time = TIME.borrow(cs).borrow_mut();

                // If we received a 0, reset the time
                if received == b'0' {
                    time.millis = 0;
                    time.seconds = 0;
                }

                *time
            });

            // Print the current time
            writeln!(serial, "{}.{:03}s\r", time.seconds, time.millis).ok();
        }
    }

    loop {
        continue;
    }
}
