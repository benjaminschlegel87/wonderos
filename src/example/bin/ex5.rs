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
use lilos::handoff::{Handoff, Pop, Push};
use stm32f3xx_hal::pac::{CorePeripherals, Peripherals};
use stm32f3xx_hal::time::fixed_point::FixedPoint;
use wonderos::l3gd20::async_spi::{async_read, async_transfer, async_write};
use wonderos::led::Led;
use wonderos::stm32f3_disco_def::{Board, GyroCs, GyroSpi, NorthEastLed, NorthLed};

const CMD: u8 = 224; // [Bit 7 = 1 (Read), Bit 6 = 1 (increment adr), Bit 5..0 = 0x20]
const WRITE_CMD: u8 = 0x20;
#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
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
        self.select_device();
        let mut enable = [WRITE_CMD, 0b0000_1111];
        async_transfer(&mut self.spi, &mut enable).await.unwrap();
        self.deselect_device()
    }
    pub async fn read_values(&mut self) -> (i16, i16, i16, i8) {
        let mut buf = [0; 14];
        self.select_device();
        // write Reg Addr
        async_write(&mut self.spi, CMD).await.unwrap();
        // clear Read Register
        let _ = async_read(&mut self.spi).await.unwrap();
        // Read Registers
        let l3gd20 = async_transfer(&mut self.spi, &mut buf).await.unwrap();
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

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut core = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();
    let mut b = Board::new(p);

    let mut transfer_x_acc: Handoff<i16> = Handoff::new();
    let (x_acc_tx, x_acc_rx) = transfer_x_acc.split();

    let g = pin!(read_gyro(b.gyro_spi, b.gyro_cs, x_acc_tx));
    let r = pin!(blink_on_acceleration(&mut b.north_led, x_acc_rx));
    let wake = pin!(wake());
    let blink = pin!(blink_parallel(&mut b.northeast_led));

    println!("Use print");

    lilos::time::initialize_sys_tick(&mut core.SYST, b.clocks.sysclk().integer());
    lilos::exec::run_tasks(&mut [g, r, blink, wake], lilos::exec::ALL_TASKS);
}
async fn read_gyro(spi: GyroSpi, cs: GyroCs, mut tx_x_acc: Push<'_, i16>) -> Infallible {
    // Enable Gyro once
    let mut gyro = L3gd20::new(spi, cs);
    gyro.enable().await;
    // Loop results every second
    loop {
        // Hole neue Messwerte vom Gyro
        let (x, _y, _z, _temp) = gyro.read_values().await;
        // Push neuen Wert zu receive Task
        // push wartet bis pop durchgeführt wurde
        tx_x_acc.push(x).await
    }
}

async fn blink_on_acceleration(led: &mut NorthLed, mut rx_x_acc: Pop<'_, i16>) -> Infallible {
    loop {
        let x = rx_x_acc.pop().await;
        if x.abs() > 20000 {
            for _i in 0..2 {
                led.on();
                lilos::exec::sleep_for(Duration::from_millis(100)).await;
                led.off();
                lilos::exec::sleep_for(Duration::from_millis(100)).await;
            }
        }
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
