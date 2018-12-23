#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use crate::hal::delay::Delay;
use crate::hal::gpio::*;
use crate::hal::prelude::*;

use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::Peripherals as c_m_Peripherals;
use cortex_m_rt::{entry, interrupt};

pub use crate::hal::stm32;
pub use crate::hal::stm32::*;

use core::cell::RefCell;
use core::ops::DerefMut;

// Make our LED globally available
static LED: Mutex<RefCell<Option<gpiob::PB3<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));

// Make our delay provider globally available
static DELAY: Mutex<RefCell<Option<Delay>>> = Mutex::new(RefCell::new(None));

// Make external interrupt registers globally available
static INT: Mutex<RefCell<Option<EXTI>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (Peripherals::take(), c_m_Peripherals::take()) {
        let gpiob = p.GPIOB.split();
        let syscfg = p.SYSCFG_COMP;
        let exti = p.EXTI;

        // Enable clock for SYSCFG
        let rcc = p.RCC;
        rcc.apb2enr.modify(|_, w| w.syscfgen().set_bit());

        // Configure PB1 as input (button)
        let _ = gpiob.pb1.into_pull_down_input();

        // Configure PB3 as output (LED)
        let mut led = gpiob.pb3.into_push_pull_output();

        // Turn off LED
        led.set_low();

        // Configure clock to 8 MHz (i.e. the default) and freeze it
        let clocks = rcc.constrain().cfgr.sysclk(8.mhz()).freeze();

        // Initialise delay provider
        let delay = Delay::new(cp.SYST, clocks);

        // Enable external interrupt for PB1
        syscfg
            .syscfg_exticr1
            .modify(|_, w| unsafe { w.exti1().bits(1) });

        // Set interrupt request mask for line 1
        exti.imr.modify(|_, w| w.mr1().set_bit());

        // Set interrupt rising trigger for line 1
        exti.rtsr.modify(|_, w| w.tr1().set_bit());

        // Move control over LED and DELAY and EXTI into global mutexes
        cortex_m::interrupt::free(move |cs| {
            *LED.borrow(cs).borrow_mut() = Some(led);
            *DELAY.borrow(cs).borrow_mut() = Some(delay);
            *INT.borrow(cs).borrow_mut() = Some(exti);
        });

        // Enable EXTI IRQ, set prio 1 and clear any pending IRQs
        let mut nvic = cp.NVIC;
        nvic.enable(Interrupt::EXTI0_1);
        unsafe { nvic.set_priority(Interrupt::EXTI0_1, 1) };
        cortex_m::peripheral::NVIC::unpend(Interrupt::EXTI0_1);
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
    cortex_m::interrupt::free(|cs| {
        // Obtain all Mutex protected resources
        if let (&mut Some(ref mut led), &mut Some(ref mut delay), &mut Some(ref mut exti)) = (
            LED.borrow(cs).borrow_mut().deref_mut(),
            DELAY.borrow(cs).borrow_mut().deref_mut(),
            INT.borrow(cs).borrow_mut().deref_mut(),
        ) {
            // Turn on LED
            led.set_high();

            // Wait a second
            delay.delay_ms(1_000_u16);

            // Turn off LED
            led.set_low();

            // Clear interrupt
            exti.pr.modify(|_, w| w.pif1().set_bit());
        }
    });
}
