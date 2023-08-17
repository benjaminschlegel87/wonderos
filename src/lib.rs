#![no_std]
/// Basic (async) Button logic
pub mod button;
/// Minimal access of the L3GD20 Accelerometer via (async) SPI
pub mod l3gd20;
/// Basic Led logic
pub mod led;
/// Minimal access of the LSM303DLHC Magneto/Accelerometer via (async) I2C
pub mod lsm303dlhc;
/// Board Defs
pub mod stm32f3_disco_def;

use core::convert::Infallible;
use core::time::Duration;
use defmt::println;
use led::Led;
use lilos::exec::{wake_tasks_by_mask, yield_cpu};
use lsm303dlhc::{Lsm303Error, Lsm303dlhc};
use stm32f3_disco_def::{EastLed, GyroScl, GyroSda, UserButton};
use stm32f3xx_hal::pac::I2C1;

/// Function performing a async debouncing of a button
///
/// Logic awaits a button press. When detected we use a select logic ([lilos::exec::with_timeout]) to check if the button is
/// release before 100ms are passed.
/// If button is not released before 100ms elapse we detect a valid button press and return true
/// else the button was released too soon and we return false
pub async fn debounced_press(but: &stm32f3_disco_def::UserButton) -> bool {
    but.wait_for_press().await;
    match lilos::exec::with_timeout(
        core::time::Duration::from_millis(200),
        but.wait_for_release(),
    )
    .await
    {
        None => {
            // valid press
            return true;
        }
        Some(_res) => {
            // invalid press
            return false;
        }
    }
}

/// Basic async task implementing logic to provide magnetometer data on button press
///
/// For every valid button press we read x,y and z-Axis information and print it
pub async fn task_magnetometer(
    mut but: UserButton,
    mut magnetometer: Lsm303dlhc<I2C1, GyroScl, GyroSda>,
) -> Infallible {
    loop {
        match magnetometer_statemachine(&mut but, &mut magnetometer).await {
            Err(err) => {
                if let Lsm303Error::Communication(e) = err {
                    panic!("I2C Communication Error with Nr {}", e as usize);
                } else {
                    panic!("Lsm303 General Error");
                }
            }
            Ok(_) => unreachable!(),
        }
    }
}

/// Internal Fallible logic
async fn magnetometer_statemachine(
    but: &mut UserButton,
    magnetometer: &mut Lsm303dlhc<I2C1, GyroScl, GyroSda>,
) -> Result<(), Lsm303Error> {
    // Write basic setup to registers
    magnetometer.setup().await?;
    loop {
        // wait for a valid press
        debounced_press(but).await;
        // get x,y and z Data
        let (x, y, z) = magnetometer.get_orientation().await?;
        println!(" x: {} y: {} z: {}", x, y, z);
        // check that button has been release to prevent perma printing for a perma pressed button
        but.wait_for_release().await;
    }
}

/// Basic Heartbeat Task - Blinks every second
pub async fn task_blinky(mut led: EastLed) -> Infallible {
    loop {
        led.toggle();
        lilos::exec::sleep_for(Duration::from_millis(1000)).await;
    }
}

/// By default [lilos] tasks must be woken by [lilos::exec::Notify] or [core::task::Waker] as the run only once until first await point
///
/// As we want just to run all tasks in a tight loop we add a task at the last spot which wake all tasks up
/// This only works if this tasks is the last in slice of tasks provided to [lilos::exec::run_tasks]
pub async fn wake() -> Infallible {
    loop {
        wake_tasks_by_mask(usize::MAX);
        yield_cpu().await
    }
}
