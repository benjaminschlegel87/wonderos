pub mod async_spi;

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;

const CMD: u8 = 224; // [Bit 7 = 1 (Read), Bit 6 = 1 (increment adr), Bit 5..0 = 0x20]
const WRITE_CMD: u8 = 0x20;

pub struct L3gd20<T: FullDuplex<u8>, E: OutputPin> {
    spi: T,
    cs: E,
}
impl<T: FullDuplex<u8>, E: OutputPin> L3gd20<T, E> {
    pub fn new(spi: T, cs: E) -> Self {
        Self { spi, cs }
    }

    pub async fn enable(&mut self) {
        self.select_device();
        let mut enable = [WRITE_CMD, 0b0000_1111];
        async_spi::async_transfer(&mut self.spi, &mut enable)
            .await
            .unwrap();
        self.deselect_device()
    }
    pub async fn read_values(&mut self) -> (i16, i16, i16, i8) {
        let mut buf = [0; 14];
        self.select_device();
        // write Reg Addr
        async_spi::async_write(&mut self.spi, CMD).await.unwrap();
        // clear Read Register
        let _ = async_spi::async_read(&mut self.spi).await.unwrap();
        // Read Registers
        let l3gd20 = async_spi::async_transfer(&mut self.spi, &mut buf)
            .await
            .unwrap();
        self.deselect_device();

        let x = i16::from_be_bytes([l3gd20[9], l3gd20[8]]);
        let y = i16::from_be_bytes([l3gd20[11], l3gd20[10]]);
        let z = i16::from_be_bytes([l3gd20[13], l3gd20[12]]);
        (x, y, z, l3gd20[6] as i8)
    }

    fn select_device(&mut self) {
        self.cs.set_low().unwrap_or_default();
    }
    fn deselect_device(&mut self) {
        self.cs.set_high().unwrap_or_default();
    }
}
