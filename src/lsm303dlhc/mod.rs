/// Async I2C Implementation: We use the clock strechting feature to avoid the use of interrupts
///
/// While the RX Data Register is not empty, the STM32 I2c Peripheral stretches the SCL Line low and does not perform further communication
/// This enables us to avoid interrupts to empty the data register. We just poll the RX Register from App Context and if we are too slow the clock
/// is stretched. This comes at the cost of maximum communication speed but enables us to implement async read/writes without the need for interrupts
pub mod i2c_no_irq;
