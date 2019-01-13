#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use crate::hal::{prelude::*, stm32};

use cortex_m::{interrupt::Mutex, peripheral::syst::SystClkSource::Core};
use cortex_m_rt::{entry, exception};

use core::{cell::RefCell, fmt::Write};

struct Shared {
    adc: hal::adc::Adc,
    tx: hal::serial::Tx<stm32::USART1>,
}

static SHARED: Mutex<RefCell<Option<Shared>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (
        hal::stm32::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        cortex_m::interrupt::free(move |cs| {
            let mut flash = p.FLASH;
            let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut flash);

            let gpioa = p.GPIOA.split(&mut rcc);

            let mut syst = cp.SYST;

            // Set source for SysTick counter, here full operating frequency (== 8MHz)
            syst.set_clock_source(Core);

            // Set reload value, i.e. timer delay 8 MHz/counts
            syst.set_reload(8_000_000 - 1);

            // Start SysTick counter
            syst.enable_counter();

            // Start SysTick interrupt generation
            syst.enable_interrupt();

            // USART1 at PA9 (TX) and PA10(RX)
            let tx = gpioa.pa9.into_alternate_af1(cs);
            let rx = gpioa.pa10.into_alternate_af1(cs);

            // Initialiase UART
            let (mut tx, _) =
                hal::serial::Serial::usart1(p.USART1, (tx, rx), 115_200.bps(), &mut rcc).split();

            // Initialise ADC
            let adc = hal::adc::Adc::new(p.ADC, &mut rcc);

            // Output a friendly greeting
            tx.write_str("\n\rThis ADC example will read various values using the ADC and print them out to the serial terminal\r\n").ok();

            // Move all components under Mutex supervision
            *SHARED.borrow(cs).borrow_mut() = Some(Shared { adc, tx });
        });
    }

    loop {
        continue;
    }
}

#[exception]
fn SysTick() -> ! {
    use core::ops::DerefMut;

    // Enter critical section
    cortex_m::interrupt::free(|cs| {
        // Get access to the Mutex protected shared data
        if let Some(ref mut shared) = SHARED.borrow(cs).borrow_mut().deref_mut() {
            // Read temperature data from internal sensor using ADC
            let t = hal::adc::VTemp::read(&mut shared.adc, None);
            writeln!(shared.tx, "Temperature {}.{}C\r", t / 100, t % 100).ok();

            // Read volatage reference data from internal sensor using ADC
            let t = hal::adc::VRef::read_vdda(&mut shared.adc);
            writeln!(shared.tx, "Vdda {}mV\r", t).ok();
        }
    });
}
