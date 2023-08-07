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
use nb::block;
use stm32f3xx_hal::pac::{CorePeripherals, Peripherals};
use stm32f3xx_hal::prelude::_embedded_hal_blocking_spi_Transfer;
use stm32f3xx_hal::time::fixed_point::FixedPoint;
use wonderos::async_spi::{async_read, async_transfer, async_write};
use wonderos::led::Led;
use wonderos::stm32f3_disco_def::{Board, GyroCs, GyroSpi, NorthEastLed};
#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}

const CMD: u8 = 143; // [Bit 7 = 1 (Read), Bit 6 = 0 (Not increment adr), Bit 5..0 = 0xF (adr who am i)]

fn blocking_who_am_i(spi: &mut GyroSpi, cs: &mut GyroCs) {
    let _ = cs.set_low();
    println!("Cmd {}", CMD);
    block!(spi.send(CMD)).unwrap();
    let res = block!(spi.read()).unwrap();
    println!("Res {}", res);
    block!(spi.send(0 as u8)).unwrap();
    let res = block!(spi.read()).unwrap();
    println!("Res {}", res);

    let _ = cs.set_high();
}

fn transfer_blocking_who_am_i(spi: &mut GyroSpi, cs: &mut GyroCs) {
    let _ = cs.set_low();
    let mut payload = [CMD, 0];
    let r = spi.transfer(&mut payload).unwrap();
    println!(" other res {}", r);
    let _ = cs.set_high();
}
#[cortex_m_rt::entry]
fn main() -> ! {
    let mut core = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();
    let mut b = Board::new(p);
    let mut spi = b.gyro_spi;
    let mut cs = b.gyro_cs;

    // Read Who Am I Register with Trait Methods from embedded-hal read and send + nb crate block! macro
    blocking_who_am_i(&mut spi, &mut cs);
    // Read Who Am I Register with Trait Method from stm32f3xx_hal::prelude::embedded_hal_blocking_spi_transfer => transfer function
    transfer_blocking_who_am_i(&mut spi, &mut cs);

    let read = pin!(read_who_am_i_async(&mut spi, &mut cs));
    let wake = pin!(wake());
    let blink = pin!(blink_parallel(&mut b.northeast_led));

    lilos::time::initialize_sys_tick(&mut core.SYST, b.clocks.sysclk().integer());
    lilos::exec::run_tasks(&mut [read, blink, wake], lilos::exec::ALL_TASKS);
}

async fn read_who_am_i_async(spi: &mut GyroSpi, cs: &mut GyroCs) -> Infallible {
    let mut once = false;
    loop {
        // Here we run this once
        if once == false {
            // Once by our own async write and read implementation
            cs.set_low().expect("");
            async_write(spi, CMD).await.unwrap();
            async_read(spi).await.unwrap();
            async_write(spi, 0 as u8).await.unwrap();
            println!("Who am I is 0b{:b}", async_read(spi).await.unwrap());
            cs.set_high().expect("");
            lilos::exec::sleep_for(Duration::from_millis(1000)).await;

            // And once with our async transfer impl
            cs.set_low().expect("");
            let mut buf = [CMD, 0];
            let res = async_transfer(spi, &mut buf).await.unwrap();
            println!("Other who Am I {}", res);
            cs.set_high().expect("");
            lilos::exec::sleep_for(Duration::from_millis(1000)).await;
            once = true;
        } else {
            // just yield after running once
            lilos::exec::yield_cpu().await
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
