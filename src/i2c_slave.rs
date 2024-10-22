pub use crate::i2c;
use crate::i2c::{SclPin, SdaPin};
use crate::rcc::Rcc;
use core::ops::Deref;
const BUFFER_SIZE: usize = 32;

/// I2C slave state
/// DataRequested: Data requested by the master, 8bit register address attached
///
/// DataReceived: Data received from the master, 8bit register address attached
///
/// Buzy: I2C interrupt that is currently being handled
///
#[derive(Copy, Clone, Debug)]
pub enum State {
    DataRequested(u8),
    DataReceived(u8),
    Buzy(I2CInterrupt),
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum TransferState {
    Idle,
    Addr(Direction),
    RegSet,
    Receiving,
    Transmitting,
}
/// I2C errors
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Overrun/Underrun
    OVERRUN,
    /// Not Acknowledge received
    NACK,
    /// Bus error
    BUS,
    /// Arbitration lost
    ARBITRATION,
    /// Timeout / t LOW error
    TIMEOUT,
    /// Packet error checking
    PEC,
    /// Unknown error
    UNKNOWN,
}
impl Into<Error> for I2CInterrupt {
    fn into(self) -> Error {
        match self {
            I2CInterrupt::Overrun => Error::OVERRUN,
            I2CInterrupt::NotAcknowledgeReceived => Error::NACK,
            I2CInterrupt::BusError => Error::BUS,
            I2CInterrupt::ArbitrationLost => Error::ARBITRATION,
            I2CInterrupt::Timeout => Error::TIMEOUT,
            _ => Error::UNKNOWN,
        }
    }
}
/// I2C slave
/// # Example
/// ```rust
/// use cortex_m::{interrupt::Mutex, peripheral::Peripherals as c_m_Peripherals};
/// static I2C_ADDR: u8 = 0x55;
/// if let (Some(mut p), Some(cp)) = (Peripherals::take(), c_m_Peripherals::take()) {
///     cortex_m::interrupt::free(move |cs| {
///         let gpioa = p.GPIOA.split(&mut rcc);
///         let sda = gpioa.pa10.into_alternate_af4(cs);
///         let scl = gpioa.pa9.into_alternate_af4(cs);
///         let i2c = i2c_slave::I2CSlave::i2c1_slave(p.I2C1, (scl, sda), I2C_ADDR, &mut rcc);
///     });
/// }
///
/// ```
///
pub struct I2CSlave<I2C, SCL, SDA> {
    i2c: I2C,
    transfer_buffer: [u8; BUFFER_SIZE],
    transfer_len: usize,
    buffer_index: usize,
    register: u8,
    transfer_state: TransferState,
    pins: (SCL, SDA),
}

/// direction as specified in the datasheet
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Direction {
    /// slave is receiver
    Write,
    /// slave is transmitter
    Read,
}

impl From<Direction> for bool {
    fn from(dir: Direction) -> Self {
        Direction::Read == dir
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

macro_rules! i2c_slave {
    ($($I2C:ident: ($i2c_slave:ident, $i2cXen:ident, $i2cXrst:ident, $apbenr:ident, $apbrstr:ident),)+) => {
        $(
            use crate::pac::$I2C;
            impl<SCLPIN, SDAPIN> I2CSlave<$I2C, SCLPIN, SDAPIN> {
                pub fn $i2c_slave(i2c: $I2C, pins: (SCLPIN, SDAPIN), address: u8, rcc: &mut Rcc) -> Self
                where
                    SCLPIN: SclPin<$I2C>,
                    SDAPIN: SdaPin<$I2C>,
                {
                    // Enable clock for I2C
                    rcc.regs.$apbenr.modify(|_, w| w.$i2cXen().set_bit());

                    // Reset I2C
                    rcc.regs.$apbrstr.modify(|_, w| w.$i2cXrst().set_bit());
                    rcc.regs.$apbrstr.modify(|_, w| w.$i2cXrst().clear_bit());
                    I2CSlave {
                        i2c,
                        transfer_buffer: [0u8; BUFFER_SIZE],
                        transfer_len: 0,
                        buffer_index: 0,
                        register: 0,
                        transfer_state: TransferState::Idle,
                        pins
                    }.i2c_init(address)
                }
            }
        )+
    }
}

/// Enum representing the diffrent interuptt triggers from the STM32 reference manual:
/// The I2C slave interrupt request is generated when the following events occur:

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum I2CInterrupt {
    /// • Receive buffer not empty (RXNE = 1)
    ReceiveBufferNotEmpty,
    /// • Transmit buffer interrupt status (TXIS = 1)
    TXBufIntStatus,
    /// • Stop detection (STOPF = 1)
    StopDetection,
    /// • Transfer complete reload (TCR = 1)
    TransferCompleteReload,
    /// • Transfer complete (TC = 1)
    TransferComplete,
    /// • Address matched with own address (ADDR = 1)
    AddressMatch(Direction),
    /// • Bus error (BERR = 1)
    BusError,
    /// • Arbitration lost (ARLO = 1)
    ArbitrationLost,
    /// • Overrun/Underrun (OVR = 1)
    Overrun,
    /// • Timeout / t LOW error  (TIMEOUT = 1)
    Timeout,
    /// • SMBus alert (ALERT = 1)
    SMBusAlert,
    /// • Not Acknowledge received (NACKF = 1)
    NotAcknowledgeReceived,
}
i2c_slave! {
    I2C1: (i2c1_slave, i2c1en, i2c1rst, apb1enr, apb1rstr),
}

#[cfg(any(
    feature = "stm32f030x8",
    feature = "stm32f030xc",
    feature = "stm32f051",
    feature = "stm32f058",
    feature = "stm32f070xb",
    feature = "stm32f071",
    feature = "stm32f072",
    feature = "stm32f078",
    feature = "stm32f091",
    feature = "stm32f098",
))]
i2c_slave! {
    I2C2: (i2c2_slave, i2c2en, i2c2rst, apb1enr, apb1rstr),
}
// It's s needed for the impls, but rustc doesn't recognize that
#[allow(dead_code)]
type I2cRegisterBlock = crate::pac::i2c1::RegisterBlock;

impl<I2C, SCL, SDA> I2CSlave<I2C, SCL, SDA>
where
    I2C: Deref<Target = I2cRegisterBlock>,
{
    /// Function to be called from the interrupt handler
    /// Returns DataRequested, DataReceived, Buzy or an Error
    ///
    /// # Example
    /// ```rust
    /// #[interrupt]
    /// fn I2C1() {
    ///     static mut I2C: Option<I2CSlave<I2C, SCL, SDA>> = None;
    ///     let i2c = I2C.get_or_insert_with(|| {
    ///         cortex_m::interrupt::free(|cs| {
    ///             // Move I2C pin here, leaving a None in its place
    ///             GI2C.borrow(cs).replace(None).unwrap()
    ///         })
    ///     });
    ///     match i2c.interrupt() {
    ///         Ok(State::Buzy(flag)) => {
    ///             rprintln!("I2C is busy {:?}", flag);
    ///         }
    ///         Ok(State::DataReceived(reg)) => {
    ///             let data = i2c.get_received_data();
    ///             rprintln!("Reg: {:?} Data: {:?}", reg, data);
    ///         }
    ///         Ok(State::DataRequested(reg)) => {
    ///             rprintln!("Data requested: {:?}", reg);
    ///             if let Err(e) = i2c.send_data(Some(&[0x01, 0x02, 0x03])) {
    ///                 rprintln!("Error {:?}", e);
    ///             }
    ///         }
    ///         Err(e) => {
    ///             rprintln!("Error {:?}", e);
    ///         }
    ///     }
    /// }
    /// ```
    ///
    pub fn interrupt(&mut self) -> Result<State, Error> {
        let mut ret = Err(Error::UNKNOWN);
        if let Some(interrupt_flag) = self.get_interrupt() {
            use I2CInterrupt as I;
            use TransferState as TS;
            ret = match (interrupt_flag, self.transfer_state) {
                (I::AddressMatch(dir), TS::Idle) => {
                    self.transfer_state = TS::Addr(dir);
                    self.clear_interrupt(interrupt_flag);
                    if dir == Direction::Read {
                        // enable TXIE to avoid deadlock when master is reading without setting register first
                        self.txie(true);
                        self.transfer_state = TS::Transmitting;
                        self.write(0xff);
                    }
                    Ok(State::Buzy(interrupt_flag))
                }
                (I::NotAcknowledgeReceived, TS::Addr(Direction::Read) | TS::Transmitting) => {
                    self.clear_interrupt(interrupt_flag);
                    self.reset_i2c();
                    Ok(State::Buzy(interrupt_flag))
                }
                (I::AddressMatch(Direction::Read), TS::RegSet) => {
                    self.transfer_state = TS::Transmitting;
                    self.txie(true);
                    self.flush_txdr();
                    Ok(State::DataRequested(self.register))
                }
                (I::ReceiveBufferNotEmpty, TS::Addr(Direction::Write)) => {
                    self.transfer_state = TS::RegSet;
                    self.register = self.read();
                    Ok(State::Buzy(interrupt_flag))
                }
                (I::ReceiveBufferNotEmpty, TS::RegSet | TS::Receiving) => {
                    if self.buffer_index >= BUFFER_SIZE - 1 {
                        self.send_nack();
                        self.clear_interrupt(interrupt_flag);
                        return Err(Error::OVERRUN);
                    }
                    self.transfer_state = TS::Receiving;
                    self.transfer_buffer[self.buffer_index] = self.read();
                    self.buffer_index += 1;
                    if self.stopcf() {
                        self.transfer_state = TS::Idle;
                        self.clear_interrupt(I::StopDetection);
                        Ok(State::DataReceived(self.register))
                    } else {
                        Ok(State::Buzy(interrupt_flag))
                    }
                }
                (I::StopDetection, _) => {
                    let ret = match self.transfer_state {
                        TS::Receiving => Ok(State::DataReceived(self.register)),
                        TS::Transmitting => Ok(State::Buzy(interrupt_flag)),
                        TS::RegSet => Ok(State::Buzy(interrupt_flag)),
                        _ => Err(Error::BUS),
                    };
                    self.transfer_state = TS::Idle;
                    self.clear_interrupt(I::StopDetection);
                    self.reset_i2c();
                    self.txie(false);
                    ret
                }

                (I::TXBufIntStatus, TS::Transmitting) if self.buffer_index <= self.transfer_len => {
                    if self.buffer_index < self.transfer_len {
                        self.write(self.transfer_buffer[self.buffer_index]);
                    } else if self.buffer_index == self.transfer_len {
                        self.write(0xff); // need to write to end the clock stretching
                    }
                    self.buffer_index += 1;
                    Ok(State::Buzy(interrupt_flag))
                }
                (I::TXBufIntStatus, _) => {
                    self.write(0xff); // need to write to end the clock stretching
                    Err(Error::OVERRUN)
                }
                _ => {
                    self.transfer_state = TS::Idle;
                    self.clear_interrupt(interrupt_flag);
                    self.reset_i2c();
                    Err(interrupt_flag.into())
                }
            }
        }
        ret
    }

    /// Send data to the I2C buffer and start the transmission
    pub fn send_data(&mut self, buffer: Option<&[u8]>) -> Result<(), Error> {
        match buffer {
            Some(data) if data.len() > BUFFER_SIZE => {
                self.send_nack();
                self.clear_interrupt(I2CInterrupt::AddressMatch(Direction::Write));
                self.write(0xff);
                Err(Error::OVERRUN)
            }
            None => {
                self.send_nack();
                self.clear_interrupt(I2CInterrupt::AddressMatch(Direction::Write));
                self.write(0xff);
                Ok(())
            }
            Some(data) => {
                for (index, item) in data.iter().enumerate() {
                    self.transfer_buffer[index] = *item;
                }
                self.transfer_len = data.len();
                self.buffer_index = 0;
                self.transfer_state = TransferState::Transmitting;
                self.write(self.transfer_buffer[self.buffer_index]);
                self.buffer_index += 1;
                self.clear_interrupt(I2CInterrupt::AddressMatch(Direction::Write));
                Ok(())
            }
        }
    }

    /// Get the received data from the I2C peripheral
    pub fn get_received_data(&mut self) -> &[u8] {
        let data = &self.transfer_buffer[..self.buffer_index];
        self.buffer_index = 0;
        self.transfer_len = 0;
        data
    }

    /// Release the I2C peripheral and pins
    pub fn release(self) -> (I2C, (SCL, SDA)) {
        (self.i2c, self.pins)
    }
    fn send_nack(&self) {
        self.i2c.cr2.modify(|_, w| w.nack().set_bit());
    }
    fn i2c_init(self, address: u8) -> Self {
        self.i2c.cr1.write(|w| {
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
        self.txie(false);
        self.i2c.oar1.write(|w| {
            w.oa1en()
                .enabled()
                .oa1()
                .bits((address as u16) << 1)
                .oa1mode()
                .bit7()
        });

        self.i2c.cr1.modify(
            |_, w| w.pe().enabled(), // enable peripheral
        );
        self
    }

    fn clear_interrupt(&self, interrupt: I2CInterrupt) {
        use I2CInterrupt as I;
        match interrupt {
            I::ReceiveBufferNotEmpty => {
                let _ = self.read();
            }
            I::TXBufIntStatus => {
                self.write(0xff);
            }
            I::StopDetection => {
                self.i2c.icr.write(|w| w.stopcf().clear());
            }
            I::TransferComplete | I::TransferCompleteReload => {
                // Only in master mode, do nothing
            }
            I::AddressMatch(_dir) => {
                self.i2c.icr.write(|w| w.addrcf().clear());
            }
            I::BusError => {
                self.i2c.icr.write(|w| w.berrcf().clear());
            }
            I::ArbitrationLost => {
                self.i2c.icr.write(|w| w.arlocf().clear());
            }
            I::Overrun => {
                self.i2c.icr.write(|w| w.ovrcf().clear());
            }
            I::Timeout => {
                self.i2c.icr.write(|w| w.timoutcf().clear());
            }
            I::SMBusAlert => {
                self.i2c.icr.write(|w| w.alertcf().clear());
            }
            I::NotAcknowledgeReceived => {
                self.i2c.icr.write(|w| w.nackcf().clear());
            }
        }
    }
    fn stopcf(&self) -> bool {
        self.i2c.isr.read().stopf().bit_is_set()
    }

    // find what triggered the interrupt
    fn get_interrupt(&self) -> Option<I2CInterrupt> {
        let isr = self.i2c.isr.read();
        use I2CInterrupt as I;
        if isr.rxne().bit_is_set() {
            return Some(I::ReceiveBufferNotEmpty);
        }

        if isr.tcr().bit_is_set() {
            return Some(I::TransferCompleteReload);
        }
        if isr.tc().bit_is_set() {
            return Some(I::TransferComplete);
        }
        if isr.addr().bit_is_set() {
            return Some(I::AddressMatch(Direction::from(isr.dir().bit())));
        }
        if isr.berr().bit_is_set() {
            return Some(I::BusError);
        }
        if isr.arlo().bit_is_set() {
            return Some(I::ArbitrationLost);
        }
        if isr.ovr().bit_is_set() {
            return Some(I::Overrun);
        }
        if isr.timeout().bit_is_set() {
            return Some(I::Timeout);
        }
        if isr.alert().bit_is_set() {
            return Some(I::SMBusAlert);
        }
        if isr.nackf().bit_is_set() {
            return Some(I::NotAcknowledgeReceived);
        }
        if isr.stopf().bit_is_set() {
            return Some(I::StopDetection);
        }
        if isr.txis().bit_is_set() {
            return Some(I::TXBufIntStatus);
        }
        None
    }

    fn reset_i2c(&mut self) {
        self.transfer_len = 0;
        self.buffer_index = 0;
    }

    fn flush_txdr(&mut self) {
        self.i2c.isr.modify(|_, w| w.txe().set_bit());
    }
    /// Read from the RXDR register
    fn read(&self) -> u8 {
        self.i2c.rxdr.read().bits() as u8
    }

    fn write(&self, value: u8) {
        self.i2c.txdr.write(|w| w.txdata().bits(value));
    }

    /// Enable or disable the TX interrupt

    fn txie(&self, enable: bool) {
        self.i2c
            .cr1
            .modify(|_, w| w.txie().bit(enable).tcie().bit(enable));
    }
}
