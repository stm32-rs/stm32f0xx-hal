#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal as hal;

use crate::hal::{
    delay::Delay,
    gpio::*,
    pac::{interrupt, Interrupt, Peripherals, EXTI},
    prelude::*,
};

use bare_metal::Mutex;
use cortex_m::peripheral::Peripherals as c_m_Peripherals;
use cortex_m_rt::entry;

use core::{cell::RefCell, ops::DerefMut};

// Make our LED globally available
static LED: Mutex<RefCell<Option<gpioa::PA1<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));

// Make our delay provider globally available
static DELAY: Mutex<RefCell<Option<Delay>>> = Mutex::new(RefCell::new(None));

// Make external interrupt registers globally available
static INT: Mutex<RefCell<Option<EXTI>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (Peripherals::take(), c_m_Peripherals::take()) {
        cortex_m::interrupt::free(move |_| {
            // SAFETY: We are in a critical section, but the `cortex_m` critical section
            // token is not compatible with the `bare_metal` token. Future version of the
            // `cortex_m` crate will not supply *any* token to this callback!
            let cs = unsafe { &bare_metal::CriticalSection::new() };

            // Enable clock for SYSCFG
            let rcc = p.RCC;
            rcc.apb2enr.modify(|_, w| w.syscfgen().set_bit());

            let mut flash = p.FLASH;
            let mut rcc = rcc.configure().sysclk(8.mhz()).freeze(&mut flash);

            let gpioa = p.GPIOA.split(&mut rcc);
            let gpiob = p.GPIOB.split(&mut rcc);
            let syscfg = p.SYSCFG;
            let exti = p.EXTI;

            // Configure PB1 as input (button)
            let _ = gpiob.pb1.into_pull_down_input(cs);

            // Configure PA1 as output (LED)
            let mut led = gpioa.pa1.into_push_pull_output(cs);

            // Turn off LED
            led.set_low().ok();

            // Initialise delay provider
            let delay = Delay::new(cp.SYST, &rcc);

            // Enable external interrupt for PB1
            syscfg.exticr1.modify(|_, w| unsafe { w.exti1().bits(1) });

            // Set interrupt request mask for line 1
            exti.imr.modify(|_, w| w.mr1().set_bit());

            // Set interrupt rising trigger for line 1
            exti.rtsr.modify(|_, w| w.tr1().set_bit());

            // Move control over LED and DELAY and EXTI into global mutexes
            *LED.borrow(*cs).borrow_mut() = Some(led);
            *DELAY.borrow(*cs).borrow_mut() = Some(delay);
            *INT.borrow(*cs).borrow_mut() = Some(exti);

            // Enable EXTI IRQ, set prio 1 and clear any pending IRQs
            let mut nvic = cp.NVIC;
            unsafe {
                nvic.set_priority(Interrupt::EXTI0_1, 1);
                cortex_m::peripheral::NVIC::unmask(Interrupt::EXTI0_1);
            }
            cortex_m::peripheral::NVIC::unpend(Interrupt::EXTI0_1);
        });
    }

    loop {
        continue;
    }
}

// Define an interupt handler, i.e. function to call when interrupt occurs. Here if our external
// interrupt trips when the button is pressed and will light the LED for a second
#[interrupt]
fn EXTI0_1() {
    // Enter critical section
    cortex_m::interrupt::free(|_| {
        // SAFETY: We are in a critical section, but the `cortex_m` critical section
        // token is not compatible with the `bare_metal` token. Future version of the
        // `cortex_m` crate will not supply *any* token to this callback!
        let cs = unsafe { bare_metal::CriticalSection::new() };

        // Obtain all Mutex protected resources
        if let (&mut Some(ref mut led), &mut Some(ref mut delay), &mut Some(ref mut exti)) = (
            LED.borrow(cs).borrow_mut().deref_mut(),
            DELAY.borrow(cs).borrow_mut().deref_mut(),
            INT.borrow(cs).borrow_mut().deref_mut(),
        ) {
            // Turn on LED
            led.set_high().ok();

            // Wait a second
            delay.delay_ms(1_000_u16);

            // Turn off LED
            led.set_low().ok();

            // Clear event triggering the interrupt
            exti.pr.write(|w| w.pif1().set_bit());
        }
    });
}
