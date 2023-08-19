//! Interrupt Driven Serial Echo Example
//! For NUCLEO-F031K6

#![no_main]
#![no_std]
#![deny(unsafe_code)]
#![allow(non_camel_case_types)]

use core::cell::RefCell;
use nb::block;
use panic_halt as _;

use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;

use hal::{
    delay::Delay,
    gpio::{
        gpioa::{PA15, PA2},
        Alternate, AF1,
    },
    pac::{self, interrupt, Interrupt, USART1},
    prelude::*,
    serial::Serial,
};
use stm32f0xx_hal as hal;

type SERIAL_PORT = Serial<USART1, PA2<Alternate<AF1>>, PA15<Alternate<AF1>>>;

/*
Create our global variables:

We use a Mutex because Mutexes require a CriticalSection
context in order to be borrowed. Since CriticalSection
contexts cannot overlap (by definition) we can rest assured
that the resource inside the Mutex will not violate
the RefMut's runtime borrowing rules (Given that we do not
try to borrow the RefMut more than once at a time).
*/
static GSERIAL: Mutex<RefCell<Option<SERIAL_PORT>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let (mut delay, mut led) = cortex_m::interrupt::free(|cs| {
        let dp = pac::Peripherals::take().unwrap(); // might as well panic if this doesn't work
        let cp = cortex_m::peripheral::Peripherals::take().unwrap();
        let mut flash = dp.FLASH;
        let mut rcc = dp.RCC.configure().sysclk(48.mhz()).freeze(&mut flash);

        let gpioa = dp.GPIOA.split(&mut rcc);
        let gpiob = dp.GPIOB.split(&mut rcc);

        let delay = Delay::new(cp.SYST, &rcc);

        // setup UART
        let (tx, rx) = (
            gpioa.pa2.into_alternate_af1(cs),
            gpioa.pa15.into_alternate_af1(cs),
        );

        // initialize global serial
        *GSERIAL.borrow(cs).borrow_mut() =
            Some(Serial::usart1(dp.USART1, (tx, rx), 9_600.bps(), &mut rcc));

        if let Some(ser) = GSERIAL.borrow(cs).borrow_mut().as_mut() {
            ser.listen(hal::serial::Event::Rxne); // trigger the USART1 interrupt when bytes are available (receive buffer not empty)
        }

        let led = gpiob.pb3.into_push_pull_output(cs);

        (delay, led)
    });

    #[allow(unsafe_code)] // just this once ;)
    unsafe {
        cortex_m::peripheral::NVIC::unmask(Interrupt::USART1);
    }

    loop {
        led.toggle().ok();

        delay.delay_ms(1_000u16);
    }
}

#[interrupt]
fn USART1() {
    static mut SERIAL: Option<SERIAL_PORT> = None;

    /*
    Once the main function has initialized the serial port,
    we move it into this interrupt handler, giving
    it exclusive access to the serial port.
    */
    let ser = SERIAL.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            if let Some(ser) = GSERIAL.borrow(cs).take() {
                ser
            } else {
                /*
                This means the main function failed to initialize
                the serial port.

                For this example, we will panic.
                */
                panic!();
            }
        })
    });

    if let Ok(data) = block!(ser.read()) {
        block!(ser.write(data)).ok();
    } else {
        /*
        Failed to read a byte:

        There could be some kind of alignment error or the UART
        was disconnected or something.
        */
    }
}
