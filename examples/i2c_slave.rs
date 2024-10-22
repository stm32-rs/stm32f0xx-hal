#![no_main]
#![no_std]

use crate::hal::{
    gpio::*,
    pac::{interrupt, Interrupt, Peripherals},
};
use cortex_m_rt::entry;
use panic_halt as _;

use rtt_target::{rprintln, rtt_init_print};
use stm32f0xx_hal::i2c_slave::{self, I2CSlave, State};
use stm32f0xx_hal::{self as hal, prelude::*};

use core::cell::RefCell;
use cortex_m::{interrupt::Mutex, peripheral::Peripherals as c_m_Peripherals};
type SCL = gpioa::PA9<Alternate<AF4>>;
type SDA = gpioa::PA10<Alternate<AF4>>;
type I2C = hal::pac::I2C1;
// Make I2C pin globally available
static GI2C: Mutex<RefCell<Option<I2CSlave<I2C, SCL, SDA>>>> = Mutex::new(RefCell::new(None));

#[interrupt]
fn I2C1() {
    static mut I2C: Option<I2CSlave<I2C, SCL, SDA>> = None;

    let i2c = I2C.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move I2C pin here, leaving a None in its place
            GI2C.borrow(cs).replace(None).unwrap()
        })
    });
    match i2c.interrupt() {
        Ok(State::Buzy(flag)) => {
            rprintln!("I2C is busy {:?}", flag);
        }
        Ok(State::DataReceived(reg)) => {
            let data = i2c.get_received_data();
            rprintln!("Reg: {:?} Data: {:?}", reg, data);
        }
        Ok(State::DataRequested(reg)) => {
            rprintln!("Data requested: {:?}", reg);

            if let Err(e) = i2c.send_data(Some(&[0x01, 0x02, 0x03])) {
                rprintln!("Error {:?}", e);
            }
        }
        Err(e) => {
            rprintln!("Error {:?}", e);
        }
    }
}

static I2C_ADDR: u8 = 0x52;
#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Starting I2C Slave example...");
    if let (Some(mut p), Some(cp)) = (Peripherals::take(), c_m_Peripherals::take()) {
        cortex_m::interrupt::free(move |cs| {
            let mut rcc = p
                .RCC
                .configure()
                .sysclk(48.mhz())
                .pclk(24.mhz())
                .freeze(&mut p.FLASH);
            let gpioa = p.GPIOA.split(&mut rcc);

            // Configure pins for I2C
            let sda = gpioa.pa10.into_alternate_af4(cs);
            let scl = gpioa.pa9.into_alternate_af4(cs);
            let i2c = i2c_slave::I2CSlave::i2c1_slave(p.I2C1, (scl, sda), I2C_ADDR, &mut rcc);
            *GI2C.borrow(cs).borrow_mut() = Some(i2c);

            // Enable I2C IRQ, set prio 1 and clear any pending IRQs
            let mut nvic = cp.NVIC;
            unsafe {
                nvic.set_priority(Interrupt::I2C1, 1);
                cortex_m::peripheral::NVIC::unmask(Interrupt::I2C1);
            }

            cortex_m::peripheral::NVIC::unpend(Interrupt::I2C1);
        });
    }

    loop {
        continue;
    }
}
