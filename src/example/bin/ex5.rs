#![no_main]
#![no_std]
use core::convert::Infallible;
use core::panic::PanicInfo;
use core::pin::pin;
use core::time::Duration;
use defmt::println;
use defmt_rtt as _;
use lilos::handoff::{Handoff, Pop, Push};
use stm32f3xx_hal::pac::{CorePeripherals, Peripherals};
use stm32f3xx_hal::time::fixed_point::FixedPoint;
use wonderos::l3gd20::L3gd20;
use wonderos::led::Led;
use wonderos::stm32f3_disco_def::{Board, GyroCs, GyroSpi, NorthEastLed, NorthLed};

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
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
