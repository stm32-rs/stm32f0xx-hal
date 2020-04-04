//! Touch sense controller
//!
//! From STM32 (https://www.st.com/content/ccc/resource/technical/document/application_note/9d/be/03/8c/5d/8c/49/50/DM00088471.pdf/files/DM00088471.pdf/jcr:content/translations/en.DM00088471.pdf):
//!
//! The Cs capacitance is a key parameter for sensitivity. For touchkey sensors, the Cs value is
//! usually comprised between 8.7nF to 22nF. For linear and rotary touch sensors, the value is
//! usually comprised between 47nF and 100nF. These values are given as reference for an
//! electrode fitting a human finger tip size across a few millimeters dielectric panel.

use crate::gpio::{gpioa, gpiob, Alternate, AF3};
use crate::pac::TSC;
use crate::rcc::Rcc;

#[derive(Debug)]
pub enum Event {
    /// Max count error
    MaxCountError,
    /// End of acquisition
    EndOfAcquisition,
}

#[derive(Debug)]
pub enum Error {
    /// Max count error
    MaxCountError,
    /// Wrong GPIO for reading
    InvalidPin,
}

pub trait TscPin<TSC> {
    type GROUP;
    type OFFSET;

    /// Returns the group a pin belongs to
    fn group() -> Self::GROUP;

    /// Returns the offset of the pin within the control registers
    fn offset() -> Self::OFFSET;
}

macro_rules! tsc_pins {
    ($($pin:ty => ($group:expr,$offset:expr)),+ $(,)*) => {
        $(
            impl TscPin<TSC> for $pin {
                type GROUP = u8;
                type OFFSET = u8;

                fn group() -> u8 { $group }
                fn offset() -> u8 { $offset }
            }
        )+
    };
}

tsc_pins!(
    gpioa::PA0<Alternate<AF3>> => (1_u8, 1_u8),
    gpioa::PA1<Alternate<AF3>> => (1_u8, 2_u8),
    gpioa::PA2<Alternate<AF3>> => (1_u8, 3_u8),
    gpioa::PA3<Alternate<AF3>> => (1_u8, 4_u8),
);

tsc_pins!(
    gpioa::PA4<Alternate<AF3>> => (2_u8, 1_u8),
    gpioa::PA5<Alternate<AF3>> => (2_u8, 2_u8),
    gpioa::PA6<Alternate<AF3>> => (2_u8, 3_u8),
    gpioa::PA7<Alternate<AF3>> => (2_u8, 4_u8),
);

tsc_pins!(
    gpiob::PB0<Alternate<AF3>> => (3_u8, 2_u8),
    gpiob::PB1<Alternate<AF3>> => (3_u8, 3_u8),
    gpiob::PB2<Alternate<AF3>> => (3_u8, 4_u8),
);

tsc_pins!(
    gpioa::PA9<Alternate<AF3>> => (4_u8, 1_u8),
    gpioa::PA10<Alternate<AF3>> => (4_u8, 2_u8),
    gpioa::PA11<Alternate<AF3>> => (4_u8, 3_u8),
    gpioa::PA12<Alternate<AF3>> => (4_u8, 4_u8),
);

tsc_pins!(
    gpiob::PB3<Alternate<AF3>> => (5_u8, 1_u8),
    gpiob::PB4<Alternate<AF3>> => (5_u8, 2_u8),
    gpiob::PB6<Alternate<AF3>> => (5_u8, 3_u8),
    gpiob::PB7<Alternate<AF3>> => (5_u8, 4_u8),
);

pub struct Tsc {
    tsc: TSC,
}

#[derive(Debug)]
pub struct Config {
    pub clock_prescale: Option<ClockPrescaler>,
    pub max_count: Option<MaxCount>,
    pub charge_transfer_high: Option<ChargeDischargeTime>,
    pub charge_transfer_low: Option<ChargeDischargeTime>,
}

#[derive(Debug)]
pub enum ClockPrescaler {
    Hclk = 0b000,
    HclkDiv2 = 0b001,
    HclkDiv4 = 0b010,
    HclkDiv8 = 0b011,
    HclkDiv16 = 0b100,
    HclkDiv32 = 0b101,
    HclkDiv64 = 0b110,
    HclkDiv128 = 0b111,
}

#[derive(Debug)]
pub enum MaxCount {
    /// 000: 255
    U255 = 0b000,
    /// 001: 511
    U511 = 0b001,
    /// 010: 1023
    U1023 = 0b010,
    /// 011: 2047
    U2047 = 0b011,
    /// 100: 4095
    U4095 = 0b100,
    /// 101: 8191
    U8191 = 0b101,
    /// 110: 16383
    U16383 = 0b110,
}

#[derive(Debug)]
/// How many tsc cycles are spent charging / discharging
pub enum ChargeDischargeTime {
    C1 = 0b0000,
    C2 = 0b0001,
    C3 = 0b0010,
    C4 = 0b0011,
    C5 = 0b0100,
    C6 = 0b0101,
    C7 = 0b0110,
    C8 = 0b0111,
    C9 = 0b1000,
    C10 = 0b1001,
    C11 = 0b1010,
    C12 = 0b1011,
    C13 = 0b1100,
    C14 = 0b1101,
    C15 = 0b1110,
    C16 = 0b1111,
}

impl Tsc {
    /// Initialise the touch controller peripheral
    pub fn tsc(tsc: TSC, rcc: &mut Rcc, cfg: Option<Config>) -> Self {
        // Enable the peripheral clock
        rcc.regs.ahbenr.modify(|_, w| w.tscen().set_bit());
        rcc.regs.ahbrstr.modify(|_, w| w.tscrst().set_bit());
        rcc.regs.ahbrstr.modify(|_, w| w.tscrst().clear_bit());

        let config = cfg.unwrap_or(Config {
            clock_prescale: None,
            max_count: None,
            charge_transfer_high: None,
            charge_transfer_low: None,
        });

        tsc.cr.write(|w| unsafe {
            w.ctph()
                .bits(
                    config
                        .charge_transfer_high
                        .unwrap_or(ChargeDischargeTime::C2) as u8,
                )
                .ctpl()
                .bits(
                    config
                        .charge_transfer_low
                        .unwrap_or(ChargeDischargeTime::C2) as u8,
                )
                .sse()
                .set_bit()
                .ssd()
                .bits(16)
                .pgpsc()
                .bits(config.clock_prescale.unwrap_or(ClockPrescaler::HclkDiv16) as u8)
                .mcv()
                .bits(config.max_count.unwrap_or(MaxCount::U8191) as u8)
                .tsce()
                .set_bit()
        });

        // clear interrupt & flags
        tsc.icr.write(|w| w.eoaic().set_bit().mceic().set_bit());

        Tsc { tsc }
    }

    /// Set up sample group
    pub fn setup_sample_group<PIN>(&mut self, _: &mut PIN)
    where
        PIN: TscPin<TSC, GROUP = u8, OFFSET = u8>,
    {
        let bit_pos = PIN::offset() - 1 + (4 * (PIN::group() - 1));
        let group_pos = PIN::group() - 1;

        // Schmitt trigger hysteresis on sample IOs
        self.tsc
            .iohcr
            .modify(|r, w| unsafe { w.bits(r.bits() | 1 << bit_pos) });

        // Set the sampling pin
        self.tsc
            .ioscr
            .modify(|r, w| unsafe { w.bits(r.bits() | 1 << bit_pos) });

        // Set the acquisition group based on the channel pins
        self.tsc
            .iogcsr
            .modify(|r, w| unsafe { w.bits(r.bits() | 1 << group_pos) });
    }

    /// Add a GPIO for use as a channel
    pub fn enable_channel<PIN>(&self, _channel: &mut PIN)
    where
        PIN: TscPin<TSC, GROUP = u8, OFFSET = u8>,
    {
        let bit_pos = PIN::offset() - 1 + (4 * (PIN::group() - 1));

        // Set a channel pin
        self.tsc
            .ioccr
            .modify(|r, w| unsafe { w.bits(r.bits() | 1 << bit_pos) });
    }

    /// Remove a GPIO from use as a channel
    pub fn disable_channel<PIN>(&self, _channel: &mut PIN)
    where
        PIN: TscPin<TSC, GROUP = u8, OFFSET = u8>,
    {
        let bit_pos = PIN::offset() - 1 + (4 * (PIN::group() - 1));

        // Remove a channel pin
        self.tsc
            .ioccr
            .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << bit_pos)) });
    }

    /// Starts a charge acquisition
    pub fn start(&self) {
        self.clear(Event::EndOfAcquisition);
        self.clear(Event::MaxCountError);

        // Discharge the caps ready for a new reading
        self.tsc.cr.modify(|_, w| w.iodef().clear_bit());
        self.tsc.cr.modify(|_, w| w.start().set_bit());
    }

    /// Check for events on the TSC
    pub fn check_event(&self) -> Option<Event> {
        let isr = self.tsc.isr.read();
        if isr.eoaf().bit_is_set() {
            Some(Event::EndOfAcquisition)
        } else if isr.mcef().bit_is_set() {
            Some(Event::MaxCountError)
        } else {
            None
        }
    }

    /// Clear interrupt & flags
    pub fn clear(&self, event: Event) {
        match event {
            Event::EndOfAcquisition => {
                self.tsc.icr.write(|w| w.eoaic().set_bit());
            }
            Event::MaxCountError => {
                self.tsc.icr.write(|w| w.mceic().set_bit());
            }
        }
    }

    /// Blocks waiting for a acquisition to complete or for a Max Count Error
    pub fn acquire(&self) -> Result<(), Error> {
        // Start the acquisition
        self.start();

        loop {
            match self.check_event() {
                Some(Event::MaxCountError) => {
                    self.clear(Event::MaxCountError);
                    break Err(Error::MaxCountError);
                }
                Some(Event::EndOfAcquisition) => {
                    self.clear(Event::EndOfAcquisition);
                    break Ok(());
                }
                None => {}
            }
        }
    }

    /// Reads the group count register
    pub fn read<PIN>(&self, _input: &mut PIN) -> Result<u16, Error>
    where
        PIN: TscPin<TSC, GROUP = u8, OFFSET = u8>,
    {
        let bit_pos = PIN::offset() - 1 + (4 * (PIN::group() - 1));

        // Read the current channel config
        let channel = self.tsc.ioccr.read().bits();

        // Check whether one of the enabled pins was supplied
        if channel & (1 << bit_pos) != 0 {
            Ok(self.read_unchecked(PIN::group()))
        } else {
            Err(Error::InvalidPin)
        }
    }

    /// Reads the tsc group count register
    pub fn read_unchecked(&self, group: u8) -> u16 {
        match group {
            1 => self.tsc.iog1cr.read().cnt().bits(),
            2 => self.tsc.iog2cr.read().cnt().bits(),
            3 => self.tsc.iog3cr.read().cnt().bits(),
            4 => self.tsc.iog4cr.read().cnt().bits(),
            5 => self.tsc.iog5cr.read().cnt().bits(),
            6 => self.tsc.iog6cr.read().cnt().bits(),
            _ => 0,
        }
    }

    /// Enables an interrupt event
    pub fn listen(&mut self, event: Event) {
        match event {
            Event::EndOfAcquisition => {
                self.tsc.ier.modify(|_, w| w.eoaie().set_bit());
            }
            Event::MaxCountError => {
                self.tsc.ier.modify(|_, w| w.mceie().set_bit());
            }
        }
    }

    /// Disables an interrupt event
    pub fn unlisten(&self, event: Event) {
        match event {
            Event::EndOfAcquisition => {
                self.tsc.ier.modify(|_, w| w.eoaie().clear_bit());
            }
            Event::MaxCountError => {
                self.tsc.ier.modify(|_, w| w.mceie().clear_bit());
            }
        }
    }

    /// Releases the TSC peripheral
    pub fn free(self) -> TSC {
        self.tsc
    }
}
