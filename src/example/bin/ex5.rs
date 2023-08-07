#![no_main]
#![no_std]
use core::convert::Infallible;
use core::panic::PanicInfo;
use core::pin::pin;
use core::time::Duration;
use defmt::println;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use stm32f3xx_hal::pac::{CorePeripherals, Peripherals};
use stm32f3xx_hal::time::fixed_point::FixedPoint;
use wonderos::async_spi::{async_read, async_transfer, async_write};
use wonderos::led::Led;
use wonderos::stm32f3_disco_def::{Board, GyroCs, GyroSpi, NorthEastLed};

const CMD: u8 = 224; // [Bit 7 = 1 (Read), Bit 6 = 1 (increment adr), Bit 5..0 = 0x20]
const WRITE_CMD: u8 = 0x20;
#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}

#[repr(packed)]
#[derive(Debug, Clone, Copy)]
struct L3gd20Register {
    ctrl_reg1: u8,
    ctrl_reg2: u8,
    ctrl_reg3: u8,
    ctrl_reg4: u8,
    ctrl_reg5: u8,
    reference: u8,
    out_temp: i8,
    status_reg: u8,
    out_x_l: u8,
    out_x_h: u8,
    out_y_l: u8,
    out_y_h: u8,
    out_z_l: u8,
    out_z_h: u8,
}

struct L3gd20<T: FullDuplex<u8>, E: OutputPin> {
    spi: T,
    cs: E,
}
impl<T: FullDuplex<u8>, E: OutputPin> L3gd20<T, E> {
    pub fn new(spi: T, cs: E) -> Self {
        Self { spi, cs }
    }

    pub async fn enable(&mut self) {
        self.cs.set_low().unwrap_or_default();
        let mut enable = [WRITE_CMD, 0b0000_1111];
        async_transfer(&mut self.spi, &mut enable).await.unwrap();
        self.cs.set_high().unwrap_or_default();
    }
    pub async fn read_values(&mut self) -> (i16, i16, i16, i8) {
        let mut buf = [0; core::mem::size_of::<L3gd20Register>()];
        self.cs.set_low().unwrap_or_default();
        // write Reg Addr
        async_write(&mut self.spi, CMD).await.unwrap();
        // clear Read Register
        let _ = async_read(&mut self.spi).await.unwrap();
        // Read Registers
        let l3gd20 = async_transfer(&mut self.spi, &mut buf).await.unwrap();
        self.cs.set_high().unwrap_or_default();
        let x = i16::from_be_bytes([l3gd20[9], l3gd20[8]]);
        let y = i16::from_be_bytes([l3gd20[11], l3gd20[10]]);
        let z = i16::from_be_bytes([l3gd20[13], l3gd20[12]]);
        (x, y, z, l3gd20[6] as i8)
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut core = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();
    let mut b = Board::new(p);

    let g = pin!(read_gyro(b.gyro_spi, b.gyro_cs));
    let wake = pin!(wake());
    let blink = pin!(blink_parallel(&mut b.northeast_led));

    println!("Use print");

    lilos::time::initialize_sys_tick(&mut core.SYST, b.clocks.sysclk().integer());
    lilos::exec::run_tasks(&mut [g, blink, wake], lilos::exec::ALL_TASKS);
}
async fn read_gyro(spi: GyroSpi, cs: GyroCs) -> Infallible {
    // Enable Gyro once
    let mut gyro = L3gd20::new(spi, cs);
    gyro.enable().await;
    // Loop results every second
    loop {
        let (x, y, z, temp) = gyro.read_values().await;
        println!("x: {}, y: {}, z: {} and temp is {}", x, y, z, temp);
        lilos::exec::sleep_for(Duration::from_millis(1000)).await;
    }
}

async fn blink_parallel(led: &mut NorthEastLed) -> Infallible {
    loop {
        led.toggle();
        lilos::exec::sleep_for(Duration::from_millis(1000)).await
    }
}

async fn wake() -> Infallible {
    loop {
        lilos::exec::wake_tasks_by_mask(usize::MAX);
        lilos::exec::yield_cpu().await
    }
}
