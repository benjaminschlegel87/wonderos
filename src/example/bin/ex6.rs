#![no_main]
#![no_std]
use core::panic::PanicInfo;
use core::pin::pin;
use defmt_rtt as _;
use stm32f3xx_hal::pac::CorePeripherals;
use stm32f3xx_hal::pac::{Peripherals, I2C1, I2C2};
use wonderos::lsm303dlhc::i2c_no_irq::I2cNoIrq;
use wonderos::lsm303dlhc::{Lsm303dlhc, MAGNETO_ADDR};
use wonderos::stm32f3_disco_def::{Board, OtherScl, OtherSda};
use wonderos::stm32f3_disco_def::{GyroScl, GyroSda};
#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}

type GyroAsyncI2c = I2cNoIrq<I2C1, GyroScl, GyroSda>;
type OtherAsyncI2c = I2cNoIrq<I2C2, OtherScl, OtherSda>;
#[cortex_m_rt::entry]
fn main() -> ! {
    let mut core = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();

    let b = Board::new(p);

    let i2c: GyroAsyncI2c = I2cNoIrq::new(b.gyro_i2c, MAGNETO_ADDR);
    let magn = Lsm303dlhc::new(i2c);
    let _other: OtherAsyncI2c = I2cNoIrq::new(b.other_i2c, 5 as u8);

    let t = pin!(wonderos::task_blinky(b.east_led,));
    let w = pin!(wonderos::wake());
    let g = pin!(wonderos::task_magnetometer(b.user_button, magn));
    lilos::time::initialize_sys_tick(&mut core.SYST, b.clocks.sysclk().0);
    lilos::exec::run_tasks(&mut [t, g, w], lilos::exec::ALL_TASKS);
}
