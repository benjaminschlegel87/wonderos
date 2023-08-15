/// Async I2C Implementation: We use the clock strechting feature to avoid the use of interrupts
///
/// While the RX Data Register is not empty, the STM32 I2c Peripheral stretches the SCL Line low and does not perform further communication
/// This enables us to avoid interrupts to empty the data register. We just poll the RX Register from App Context and if we are too slow the clock
/// is stretched. This comes at the cost of maximum communication speed but enables us to implement async read/writes without the need for interrupts
pub mod i2c_no_irq;

use i2c_no_irq::I2cNoIrq;
use stm32f3xx_hal::i2c::{Error, Instance};

pub const ACCEL_ADDR: u8 = 0b0001_1110;
pub const MAGNETO_ADDR: u8 = 0b0001_1110;
const CRA_REG_M: u8 = 0x00;
const MR_REG_M: u8 = 0x02;

#[derive(Debug)]
pub enum Lsm303Error {
    General,
    Communication(Error),
}
impl From<Error> for Lsm303Error {
    fn from(value: Error) -> Self {
        Self::Communication(value)
    }
}
pub struct Lsm303dlhc<T: Instance, SCL, SDA> {
    i2c: I2cNoIrq<T, SCL, SDA>,
}

impl<T: Instance, SCL, SDA> Lsm303dlhc<T, SCL, SDA> {
    pub fn new(i2c: I2cNoIrq<T, SCL, SDA>) -> Self {
        Self { i2c }
    }

    pub async fn get_orientation(&mut self) -> Result<(i16, i16, i16), Lsm303Error> {
        self.i2c.write(&[0x03], false).await?;
        let mut buf = [0 as u8; 6];
        self.i2c.read(&mut buf).await?;
        let x = i16::from_be_bytes([buf[0], buf[1]]);
        let z = i16::from_be_bytes([buf[2], buf[3]]);
        let y = i16::from_be_bytes([buf[4], buf[5]]);
        if x > 10000 {
            // never happens
            Err(Lsm303Error::General)
        } else {
            Ok((x, y, z))
        }
    }
    pub async fn setup(&mut self) -> Result<(), Lsm303Error> {
        self.i2c.write(&[MR_REG_M, 0b00], true).await?;
        self.i2c.write(&[CRA_REG_M, 0b0001000], true).await?;
        Ok(())
    }
}
