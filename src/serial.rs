use lpc11uxx::*;
use embedded_hal::serial;
use crate::clock::{Clocks, USARTClock};

pub trait PinRxd {}
pub trait PinTxd {}

impl PinRxd for crate::gpio::gpio0::Pio18<crate::gpio::RXD> {}
impl PinRxd for crate::gpio::gpio1::Pio14<crate::gpio::RXD> {}
impl PinRxd for crate::gpio::gpio1::Pio26<crate::gpio::RXD> {}

impl PinTxd for crate::gpio::gpio0::Pio19<crate::gpio::RXD> {}
impl PinTxd for crate::gpio::gpio1::Pio13<crate::gpio::RXD> {}
impl PinTxd for crate::gpio::gpio1::Pio27<crate::gpio::RXD> {}

pub trait Pins {}

impl<RXD, TXD> Pins for (RXD, TXD)
where
    RXD: PinRxd,
    TXD: PinTxd,
{}

pub struct Serial<PINS> {
    usart: USART,
    _pins: PINS
}

impl<PINS: Pins> Serial<PINS> {
    pub fn new(usart: USART, pins: PINS, clocks: Clocks, mut usart_clock: USARTClock, baudrate: crate::clock::Hertz) -> Serial<PINS> {
        usart_clock.configure(clocks, clocks.main_clock_freq());

        usart.fcr_mut().write(|v| v
            .fifoen().enabled()
            .rxfifores().clear()
            .txfifores().clear());

        usart.lcr.write(|v| v
            .wls()._8_bit_character_leng()
            .sbs()._1_stop_bit()
            .pe().disabled());

        // Disable fractional divider
        usart.fdr.write(|v| unsafe { v
            .divaddval().bits(0)
            .mulval().bits(1)
        });

        let clkin = clocks.main_clock_freq();
        let div = clkin.0 / (baudrate.0 * 16);

        assert!(div < (1 << 16), "Baudrate is too damn high!");

        let divh = (div / 256) as u8;
        let divl = (div % 256) as u8;

        usart.lcr.modify(|_, v| v.dlab().enable_access_to_div());
        usart.dll_mut().write(|v| unsafe { v.dllsb().bits(divl) });
        usart.dlm_mut().write(|v| unsafe { v.dlmsb().bits(divh) });
        usart.lcr.modify(|_, v| v.dlab().disable_access_to_di());
        Serial {
            usart, _pins: pins
        }
    }
}

// TODO: BITFLAGS
#[derive(Debug)]
pub struct Error {
    lsr: u32,
}

impl<PINS: Pins> serial::Read<u8> for Serial<PINS> {
    type Error = Error;
    fn read(&mut self) -> nb::Result<u8, Error> {
        let lsr = self.usart.lsr.read();

        if lsr.oe().is_active() || lsr.pe().is_active() || lsr.fe().is_active() {
            return Err(nb::Error::Other(Error { lsr: lsr.bits() }));
        }

        if lsr.rdr().is_valid() {
            return Err(nb::Error::WouldBlock)
        }

        Ok(self.usart.rbr().read().rbr().bits())
    }
}

impl<PINS: Pins> serial::Write<u8> for Serial<PINS> {
    type Error = Error;
    fn write(&mut self, data: u8) -> nb::Result<(), Error> {
        let lsr = self.usart.lsr.read();

        if lsr.oe().is_active() || lsr.pe().is_active() || lsr.fe().is_active() {
            return Err(nb::Error::Other(Error { lsr: lsr.bits() }));
        }

        if lsr.thre().is_valid() {
            return Err(nb::Error::WouldBlock)
        }

        Ok(self.usart.thr_mut().write(|v| unsafe {v.thr().bits(data)}))
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        let lsr = self.usart.lsr.read();

        if lsr.oe().is_active() || lsr.pe().is_active() || lsr.fe().is_active() {
            return Err(nb::Error::Other(Error { lsr: lsr.bits() }));
        }

        if lsr.temt().is_empty() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}