use super::pac::can::TX;
use super::pac;
use crate::gpio::gpiob::{PB8, PB9};
use crate::gpio::{Alternate, AF4};

pub struct CANBus {
    can: stm32::CAN,
    _rx: PB8<Alternate<AF4>>,
    _tx: PB9<Alternate<AF4>>,
}

pub enum Event {
    RxMessagePending,
}

impl CANBus {
    // TODO add setting of pins the same way as done in other peripherals
    pub fn new(can: pac::CAN, rx: PB8<Alternate<AF4>>, tx: PB9<Alternate<AF4>>) -> Self {
        unsafe {
            let rcc = &(*stm32::RCC::ptr());
            rcc.apb1enr.modify(|_, w| w.canen().enabled());
            rcc.apb1rstr.modify(|_, w| w.canrst().reset());
            rcc.apb1rstr.modify(|_, w| w.canrst().clear_bit());
        }

        can.mcr.write(|w| w.sleep().clear_bit());
        can.mcr.modify(|_, w| w.inrq().set_bit());
        while !can.msr.read().inak().bit() {}
        can.mcr.modify(|_, w| {
            w.ttcm()
                .clear_bit() // no time triggered communication
                .abom()
                .set_bit() // bus automatically recovers itself after error state
                .awum()
                .set_bit() // bus is automatically waken up on message RX
                .nart()
                .clear_bit() // automatic message retransmission enabled
                .rflm()
                .clear_bit() // new RX message overwrite unread older ones
                .txfp()
                .clear_bit() // TX message priority driven by the message identifier
                .sleep()
                .clear_bit() // do not sleep
        });
        // calculated using http://www.bittiming.can-wiki.info/ for STMicroelectronics bxCAN 48 MHz clock, 87.6% sample point, SJW = 1, bitrate 250 kHz
        const TIME_SEGMENT1: u8 = 13;
        const TIME_SEGMENT2: u8 = 2;
        const RESYNC_WIDTH: u8 = 1;
        const PRESCALER: u16 = 12;
        can.btr.modify(|_, w| unsafe {
            w.silm()
                .clear_bit() // disable silent mode
                .lbkm()
                .clear_bit() // disable loopback mode
                .sjw()
                .bits(RESYNC_WIDTH - 1)
                .ts2()
                .bits(TIME_SEGMENT2 - 1)
                .ts1()
                .bits(TIME_SEGMENT1 - 1)
                .brp()
                .bits(PRESCALER - 1)
        });

        can.mcr.modify(|_, w| w.inrq().clear_bit());
        while !can.msr.read().inak().bit() {}

        can.fmr.modify(|_, w| w.finit().set_bit()); // filter init enabled
        can.fa1r.write(|w| w.fact0().clear_bit()); // filter is inactive

        can.fm1r.write(|w| w.fbm0().clear_bit()); // identifier mask mode for fbm0
        can.fs1r.write(|w| w.fsc0().set_bit()); // 32 bit scale configuration

        // const FILTER0_ID: u16 = 0x0;
        // const FILTER0_MASK: u16 = 0x00;
        // const FILTER1_ID: u16 = 0x00;
        // const FILTER1_MASK: u16 = 0x00;
        can.fb[0].fr1.write(|w| unsafe { w.bits(0) });
        can.fb[0].fr2.write(|w| unsafe { w.bits(0) });

        can.fa1r.write(|w| w.fact0().set_bit()); // filter is active
        can.fmr.modify(|_, w| w.finit().clear_bit()); // filter init disabled

        Self {
            can,
            _rx: rx,
            _tx: tx,
        }
    }

    pub fn write(&self, frame: &CANFrame) -> nb::Result<(), CANError> {
        if self.can.tsr.read().tme0().bit_is_set() {
            self.write_to_mailbox(&self.can.tx[0], frame);
            Ok(())
        } else if self.can.tsr.read().tme1().bit_is_set() {
            self.write_to_mailbox(&self.can.tx[1], frame);
            Ok(())
        } else if self.can.tsr.read().tme2().bit_is_set() {
            self.write_to_mailbox(&self.can.tx[2], frame);
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn write_to_mailbox(&self, tx: &TX, frame: &CANFrame) {
        tx.tdtr.write(|w| unsafe { w.dlc().bits(frame.dlc) });
        tx.tdlr.write(|w| unsafe {
            w.data0()
                .bits(frame.data[0])
                .data1()
                .bits(frame.data[1])
                .data2()
                .bits(frame.data[2])
                .data3()
                .bits(frame.data[3])
        });
        tx.tdhr.write(|w| unsafe {
            w.data4()
                .bits(frame.data[4])
                .data5()
                .bits(frame.data[5])
                .data6()
                .bits(frame.data[6])
                .data7()
                .bits(frame.data[7])
        });

        tx.tir.write(|w| unsafe {
            w.stid()
                .bits(frame.id)
                .ide()
                .standard()
                .rtr()
                .bit(frame.rtr)
                .txrq()
                .set_bit()
        });
    }

    pub fn read(&self) -> nb::Result<CANFrame, CANError> {
        for (i, rfr) in self.can.rfr.iter().enumerate() {
            let pending = rfr.read().fmp().bits();

            for _ in 0..pending {
                let rx = &self.can.rx[i];
                let id = rx.rir.read().stid().bits();
                let rtr = rx.rir.read().rtr().bit_is_set();
                let dlc = rx.rdtr.read().dlc().bits();

                let data0 = rx.rdlr.read().data0().bits();
                let data1 = rx.rdlr.read().data1().bits();
                let data2 = rx.rdlr.read().data2().bits();
                let data3 = rx.rdlr.read().data3().bits();
                let data4 = rx.rdhr.read().data4().bits();
                let data5 = rx.rdhr.read().data5().bits();
                let data6 = rx.rdhr.read().data6().bits();
                let data7 = rx.rdhr.read().data7().bits();

                rfr.modify(|_, w| w.rfom().release()); // release
                if rfr.read().fovr().bit_is_set() {
                    rfr.modify(|_, w| w.fovr().clear());
                }

                if rfr.read().full().bit_is_set() {
                    rfr.modify(|_, w| w.full().clear());
                }

                let frame = CANFrame {
                    id,
                    rtr,
                    dlc,
                    data: [data0, data1, data2, data3, data4, data5, data6, data7],
                };
                return Ok(frame);
            }
        }
        Err(nb::Error::WouldBlock)
    }

    pub fn listen(&self, event: Event) {
        match event {
            Event::RxMessagePending => {
                self.can
                    .ier
                    .modify(|_, w| w.fmpie0().set_bit().fmpie1().set_bit());
            }
        }
    }
}

pub enum CANError {}

#[derive(Copy, Clone, Default)]
pub struct CANFrame {
    pub id: u16,
    pub rtr: bool,
    pub dlc: u8,
    pub data: [u8; 8],
}
