#![no_main]
#![no_std]
use core::convert::Infallible;
use core::panic::PanicInfo;
use core::pin::pin;
use core::time::Duration;
use defmt::println;
use defmt_rtt as _;
use lilos::exec::sleep_for;
use lilos::exec::{wake_tasks_by_mask, yield_cpu};
use stm32f3xx_hal::i2c::{self};
use stm32f3xx_hal::pac::CorePeripherals;
use stm32f3xx_hal::pac::Peripherals;
use wonderos::led::Led;
use wonderos::lsm303dlhc::i2c_no_irq::I2cNoIrq;
use wonderos::stm32f3_disco_def::{Board, EastLed};
use wonderos::stm32f3_disco_def::{GyroScl, GyroSda};

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}

//const LSM303DLHC_ADDR_ACC: u8 = 0b0001_1001;
const LSM303DLHC_ADDR_MAGNETIC: u8 = 0b0001_1110;
pub struct Lsm303dlhc {}
type GyroAsyncI2c = I2cNoIrq<GyroScl, GyroSda>;
#[cortex_m_rt::entry]
fn main() -> ! {
    let mut core = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();

    let mut b = Board::new(p);

    let i2c: GyroAsyncI2c = I2cNoIrq::new(b.gyro_i2c, LSM303DLHC_ADDR_MAGNETIC);

    let t = pin!(task_blinky(&mut b.east_led));
    let w = pin!(wake());
    let g = pin!(task_magneto(i2c));
    lilos::time::initialize_sys_tick(&mut core.SYST, b.clocks.sysclk().0);
    lilos::exec::run_tasks(&mut [t, g, w], lilos::exec::ALL_TASKS);
}

async fn task_blinky(led: &mut EastLed) -> Infallible {
    loop {
        led.toggle();
        lilos::exec::sleep_for(Duration::from_millis(1000)).await;
    }
}
#[derive(Debug)]
enum Lsm303Error {
    General,
    Communication(i2c::Error),
}

impl From<i2c::Error> for Lsm303Error {
    fn from(value: i2c::Error) -> Self {
        Self::Communication(value)
    }
}
const CRA_REG_M: u8 = 0x00;
const MR_REG_M: u8 = 0x02;

async fn get_orientation(i2c: &mut GyroAsyncI2c) -> Result<(i16, i16, i16), Lsm303Error> {
    i2c.write(&[0x03], false).await?;
    let mut buf = [0 as u8; 6];
    i2c.read(&mut buf).await?;
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

async fn run_magneto(i2c: &mut GyroAsyncI2c) -> Result<(), Lsm303Error> {
    i2c.write(&[MR_REG_M, 0b00], true).await?;
    i2c.write(&[CRA_REG_M, 0b0001000], true).await?;
    loop {
        let (x, y, z) = get_orientation(i2c).await?;
        println! {" x: {} y: {} z: {}",x,y,z};
        sleep_for(Duration::from_millis(100)).await;
    }
}

async fn task_magneto(mut i2c: GyroAsyncI2c) -> Infallible {
    loop {
        match run_magneto(&mut i2c).await {
            Err(Lsm303Error::Communication(x)) => {
                println!("Error {:?}", x as usize);
                panic!("I2C Error");
            }
            Err(Lsm303Error::General) => println!("General"),
            Ok(_) => unreachable!(),
        }
    }
}

async fn wake() -> Infallible {
    loop {
        wake_tasks_by_mask(usize::MAX);
        yield_cpu().await
    }
}
