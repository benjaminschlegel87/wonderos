use core::future::Future;
use stm32f3xx_hal::i2c::{Error, I2c, Instance, SclPin, SdaPin};

/// We build on top of [stm32f3xx_hal::i2c::I2c] so we can reuse all the enable and clock selection stuff
/// We only implement async read/write on top
pub struct I2cNoIrq<T: Instance, SCL, SDA> {
    i2c: I2c<T, (SCL, SDA)>,
    adr: u8,
}

impl<T: Instance, SCL, SDA> I2cNoIrq<T, SCL, SDA> {
    /// Creates an Async I2C without interrupt use by consuming a HAL Instance of I2c
    ///
    /// We use the HAL I2c to reuse all the clock selection and enabling work done by the new
    /// function of the [stm32f3xx_hal::i2c::I2c] struct.
    pub fn new(i2c: I2c<T, (SCL, SDA)>, adr: u8) -> Self
    where
        SCL: SclPin<T>,
        SDA: SdaPin<T>,
    {
        Self { i2c, adr }
    }
}
struct AsyncI2cWrite<'a, T: Instance, SCL, SDA> {
    i2c: &'a mut I2cNoIrq<T, SCL, SDA>,
    buf: &'a [u8],
    cnt: u8,
}
impl<'a, T: Instance, SCL, SDA> Future for AsyncI2cWrite<'a, T, SCL, SDA> {
    type Output = Result<(), Error>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let cnt = self.cnt as usize;
        let byte = self.buf[cnt];
        let port = &mut unsafe { self.i2c.i2c.peripheral() };
        if true == port.isr.read().txis().is_empty() {
            port.txdr.write(|w| w.txdata().bits(byte));
            self.cnt += 1;
            if self.cnt == self.buf.len() as u8 {
                core::task::Poll::Ready(Ok(()))
            } else {
                core::task::Poll::Pending
            }
        } else {
            let isr = port.isr.read();
            let icr = &port.icr;

            if isr.arlo().is_lost() {
                icr.write(|w| w.arlocf().clear());
                return core::task::Poll::Ready(Err(Error::Arbitration));
            } else if isr.berr().is_error() {
                icr.write(|w| w.berrcf().clear());
                return core::task::Poll::Ready(Err(Error::Bus));
            } else if isr.nackf().is_nack() {
                while port.isr.read().stopf().is_no_stop() {}
                icr.write(|w| w.nackcf().clear());
                icr.write(|w| w.stopcf().clear());
                return core::task::Poll::Ready(Err(Error::Nack));
            } else {
                return core::task::Poll::Pending;
            }
        }
    }
}
struct AsyncI2c<'a, T: Instance, SCL, SDA> {
    i2c: &'a mut I2cNoIrq<T, SCL, SDA>,
    buf: &'a mut [u8],
    cnt: u8,
}
impl<'a, T: Instance, SCL, SDA> Future for AsyncI2c<'a, T, SCL, SDA> {
    type Output = Result<(), Error>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let port = &mut unsafe { self.i2c.i2c.peripheral() };
        if true == port.isr.read().rxne().is_not_empty() {
            let byte = port.rxdr.read().rxdata().bits();
            let cnt = self.cnt as usize;
            self.buf[cnt] = byte;
            self.cnt += 1;
            if self.cnt == self.buf.len() as u8 {
                core::task::Poll::Ready(Ok(()))
            } else {
                core::task::Poll::Pending
            }
        } else {
            let isr = port.isr.read();
            let icr = &port.icr;

            if isr.arlo().is_lost() {
                icr.write(|w| w.arlocf().clear());
                return core::task::Poll::Ready(Err(Error::Arbitration));
            } else if isr.berr().is_error() {
                icr.write(|w| w.berrcf().clear());
                return core::task::Poll::Ready(Err(Error::Bus));
            } else if isr.nackf().is_nack() {
                while port.isr.read().stopf().is_no_stop() {}
                icr.write(|w| w.nackcf().clear());
                icr.write(|w| w.stopcf().clear());
                return core::task::Poll::Ready(Err(Error::Nack));
            } else {
                return core::task::Poll::Pending;
            }
        }
    }
}

impl<T: Instance, SCL, SDA> I2cNoIrq<T, SCL, SDA> {
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        // start transfer
        let p = unsafe { self.i2c.peripheral() };

        p.cr2.modify(|_, w| {
            w.add10().bit7();
            w.sadd().bits((self.adr << 1) as u16);
            w.rd_wrn().read();
            w.start().start();
            w.nbytes().bits(buffer.len() as u8);
            w.reload().completed().autoend().automatic()
        });
        // poll isr
        AsyncI2c {
            i2c: self,
            buf: buffer,
            cnt: 0,
        }
        .await
    }
    pub async fn write(&mut self, buffer: &[u8], with_end: bool) -> Result<(), Error> {
        let p = unsafe { self.i2c.peripheral() };
        p.cr2.modify(|_, w| {
            w.add10().bit7();
            w.sadd().bits((self.adr << 1) as u16);
            w.rd_wrn().write();
            w.start().start();
            w.nbytes().bits(buffer.len() as u8);
            if with_end {
                w.reload().completed().autoend().automatic()
            } else {
                w.reload().completed().autoend().software()
            }
        });
        AsyncI2cWrite {
            i2c: self,
            buf: buffer,
            cnt: 0,
        }
        .await
    }
}
