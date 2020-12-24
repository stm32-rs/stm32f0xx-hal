use super::i2c::{SclPin, SdaPin};
use super::pac::I2C1;

#[derive(PartialEq)]
pub enum TransferState {
    Idle,
    Addressed,
    RegisterSet,
    Receiving,
    Transmitting,
}

#[derive(Copy, Clone)]
pub enum State {
    DataRequested(u8),
    DataReceived(u8),
}

const BUFFER_SIZE: usize = 32;

pub struct I2CSlave<SDA, SCL> {
    i2c: I2C1,
    transfer_buffer: [u8; BUFFER_SIZE],
    transfer_len: usize,
    buffer_index: usize,
    register: u8,
    transfer_state: TransferState,
    state: Option<State>,
    _sda: SDA,
    _scl: SCL,
}

// direction as specified in the datasheet
#[derive(PartialEq)]
pub enum Direction {
    Write, // slave is receiver
    Read,  // slave is transmitter
}

impl From<Direction> for bool {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::Write => false,
            Direction::Read => true,
        }
    }
}

impl From<bool> for Direction {
    fn from(raw: bool) -> Self {
        if raw {
            Direction::Read
        } else {
            Direction::Write
        }
    }
}

pub enum Status {
    AddressMatch(Direction),
    Busy,
    Timeout,
    Overrun,
    ArbitrationLost,
    BusError,
    TransferCompleteReload,
    Stop,
    NACKReceived,
    RxNotEmpty,
    TxDataMustBeWritten,
    TxEmpty,
}

impl<SDA, SCL> I2CSlave<SDA, SCL>
where
    SDA: SdaPin<I2C1>,
    SCL: SclPin<I2C1>,
{
    pub fn new(i2c: I2C1, address: u8, sda: SDA, scl: SCL) -> Self {
        let rcc = unsafe { &(*stm32f0xx_hal::pac::RCC::ptr()) };
        rcc.apb1enr.modify(|_, w| w.i2c1en().enabled());
        rcc.apb1rstr.modify(|_, w| w.i2c1rst().set_bit());
        rcc.apb1rstr.modify(|_, w| w.i2c1rst().clear_bit());

        i2c.cr1.write(|w| {
            w.nostretch()
                .enabled() // enable clock stretching
                .anfoff()
                .enabled() // enable analog filter
                .dnf()
                .no_filter() // disable digital filter
                .errie()
                .enabled() // error interrupt enabled
                .stopie()
                .enabled() // stop interrupt enabled
                .nackie()
                .enabled() // nack interrupt enabled
                .addrie()
                .enabled() // address match interrupt enabled
                .rxie() // rx interrupt enabled
                .enabled()
                .wupen()
                .enabled() // wake up when address match
        });

        // TODO set up timing for nostretch mode
        // let scll = cmp::max((((48_000_000 >> 1) >> 1) / KiloHertz(100).0) - 1, 255) as u8;
        // i2c.timingr.write(|w| {
        //     w.presc()
        //         .bits(1)
        //         .scldel()
        //         .bits(4)
        //         .sdadel()
        //         .bits(2)
        //         .sclh()
        //         .bits(scll - 4)
        //         .scll()
        //         .bits(scll)
        // });

        i2c.oar1.write(|w| {
            w.oa1en()
                .enabled()
                .oa1()
                .bits((address as u16) << 1)
                .oa1mode()
                .bit7()
        });

        i2c.cr1.modify(
            |_, w| w.pe().enabled(), // enable peripheral
        );

        I2CSlave {
            i2c,
            transfer_buffer: [0u8; BUFFER_SIZE],
            transfer_len: 0,
            buffer_index: 0,
            register: 0,
            transfer_state: TransferState::Idle,
            state: None,
            _sda: sda,
            _scl: scl,
        }
    }

    pub fn is_status(&self, status: Status, clear: bool) -> bool {
        let isr = self.i2c.isr.read();

        match status {
            Status::AddressMatch(dir) => {
                if isr.addr().bit_is_set() {
                    if dir != isr.dir().bit().into() {
                        return false;
                    }
                    if clear {
                        self.i2c.icr.write(|w| w.addrcf().clear());
                    }
                    return true;
                } else {
                    false
                }
            }
            Status::Busy => {
                return isr.busy().bit_is_set();
            }
            Status::Timeout => {
                if isr.timeout().bit_is_set() {
                    if clear {
                        self.i2c.icr.write(|w| w.timoutcf().clear());
                    }
                    return true;
                }
                return false;
            }
            Status::Overrun => {
                if isr.ovr().bit_is_set() {
                    if clear {
                        self.i2c.icr.write(|w| w.ovrcf().clear());
                    }
                    return true;
                }
                return false;
            }
            Status::ArbitrationLost => {
                if isr.arlo().bit_is_set() {
                    if clear {
                        self.i2c.icr.write(|w| w.arlocf().clear());
                    }
                    return true;
                }
                return false;
            }
            Status::BusError => {
                if isr.berr().bit_is_set() {
                    if clear {
                        self.i2c.icr.write(|w| w.berrcf().clear());
                    }
                    return true;
                }
                return false;
            }
            Status::TransferCompleteReload => {
                if isr.tcr().bit_is_set() {
                    if clear {
                        // defmt::error!("Cannot be cleared.");
                    }
                    return true;
                }
                return false;
            }
            Status::Stop => {
                if isr.stopf().bit_is_set() {
                    if clear {
                        self.i2c.icr.write(|w| w.stopcf().clear());
                    }
                    return true;
                }
                return false;
            }
            Status::NACKReceived => {
                if isr.nackf().bit_is_set() {
                    if clear {
                        self.i2c.icr.write(|w| w.nackcf().clear());
                    }
                    return true;
                }
                return false;
            }
            Status::RxNotEmpty => {
                if isr.rxne().bit_is_set() {
                    if clear {
                        // defmt::error!("Cannot be cleared.");
                    }
                    return true;
                }
                return false;
            }
            Status::TxDataMustBeWritten => {
                if isr.txis().bit_is_set() {
                    if clear {
                        // defmt::error!("Cannot be cleared.");
                    }
                    return true;
                }
                return false;
            }
            Status::TxEmpty => {
                if isr.txe().bit_is_set() {
                    if clear {
                        // defmt::error!("Cannot be cleared.");
                    }
                    return true;
                }
                return false;
            }
        }
    }

    pub fn read(&self) -> u8 {
        self.i2c.rxdr.read().bits() as u8
    }

    pub fn write(&self, value: u8) {
        self.i2c.txdr.write(|w| w.txdata().bits(value));
    }

    pub fn set_txe(&self) {
        self.i2c.isr.modify(|_, w| w.txe().set_bit());
    }

    pub fn enable_txie(&self, enable: bool) {
        self.i2c
            .cr1
            .modify(|_, w| w.txie().bit(enable).tcie().bit(enable));
    }

    pub fn interrupt(&mut self) {
        if self.transfer_state == TransferState::Idle {
            self.state = None;
            self.enable_txie(false);
        }

        if self.is_status(Status::BusError, true) {
            self.handle_error();
            return;
        }
        if self.is_status(Status::Overrun, true) {
            self.handle_error();
            return;
        }
        if self.is_status(Status::ArbitrationLost, true) {
            self.handle_error();
            return;
        }
        if self.is_status(Status::NACKReceived, true) {
            self.handle_error();
            return;
        }
        if self.is_status(Status::Timeout, true) {
            self.handle_error();
            return;
        }

        if self.is_status(Status::RxNotEmpty, false) {
            if self.transfer_state == TransferState::Addressed {
                self.transfer_state = TransferState::RegisterSet;
                self.register = self.read();
            } else if self.transfer_state == TransferState::RegisterSet {
                self.transfer_state = TransferState::Receiving;
            } else if self.transfer_state == TransferState::Receiving {
                // do not change state, just read
                self.transfer_buffer[self.buffer_index] = self.read();
                self.buffer_index += 1;
            }
        }

        if self.is_status(Status::Stop, true) {
            // handle reception
            if self.transfer_state == TransferState::Receiving {
                self.state = Some(State::DataReceived(self.register));
            } else if self.transfer_state == TransferState::Transmitting {
                // data was transmitted, nothing else to do
                self.state = None;
            }
            self.i2c.isr.modify(|_, w| w.txe().set_bit()); // flush txdr
            self.transfer_state = TransferState::Idle;
        }

        if self.is_status(Status::AddressMatch(Direction::Write), true) {
            self.transfer_state = TransferState::Addressed;
        }
        if self.is_status(Status::TxDataMustBeWritten, false) {
            // this may be true more times than actual data length, ignore then
            if self.transfer_state == TransferState::Transmitting {
                // state is not changed
                if self.buffer_index < self.transfer_len {
                    self.write(self.transfer_buffer[self.buffer_index]);
                    self.buffer_index += 1;
                } else {
                    self.enable_txie(false);
                    self.state = None;
                }
            }
        }
        if self.is_status(Status::AddressMatch(Direction::Read), true) {
            if self.transfer_state == TransferState::RegisterSet {
                self.enable_txie(true);
                self.transfer_state = TransferState::Transmitting;
                self.state = Some(State::DataRequested(self.register));
            }
        }
    }

    fn handle_error(&mut self) {
        self.transfer_state = TransferState::Idle;
        self.state = None;
        self.transfer_len = 0;
        self.buffer_index = 0;
        self.i2c.isr.modify(|_, w| w.txe().set_bit()); // flush txdr
    }

    pub fn set_transmit_buffer(&mut self, buffer: &[u8]) {
        for (index, item) in buffer.iter().enumerate() {
            self.transfer_buffer[index] = *item;
        }
        self.transfer_len = buffer.len();
        self.buffer_index = 0;
        self.state = None
    }

    pub fn get_received_data(&mut self) -> &[u8] {
        let data = &self.transfer_buffer[..self.buffer_index];
        self.state = None;
        self.buffer_index = 0;
        self.transfer_len = 0;
        data
    }

    pub fn get_state(&self) -> Option<State> {
        self.state
    }
}
