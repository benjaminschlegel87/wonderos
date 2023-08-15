#![no_std]
pub mod button;
pub mod l3gd20;
pub mod led;
pub mod lsm303dlhc;
pub mod stm32f3_disco_def;

use core::convert::Infallible;
use core::time::Duration;
use defmt::println;
use led::Led;
use lilos::exec::{wake_tasks_by_mask, yield_cpu};
use lsm303dlhc::{Lsm303Error, Lsm303dlhc};
use stm32f3_disco_def::{EastLed, GyroScl, GyroSda, UserButton};
use stm32f3xx_hal::pac::I2C1;
pub async fn debounced_press(but: &stm32f3_disco_def::UserButton) -> bool {
    but.wait_for_press().await;
    match lilos::exec::with_timeout(
        core::time::Duration::from_millis(100),
        but.wait_for_release(),
    )
    .await
    {
        None => {
            return true;
        }
        Some(_res) => {
            return false;
        } //
    }
}

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

async fn magnetometer_statemachine(
    but: &mut UserButton,
    magnetometer: &mut Lsm303dlhc<I2C1, GyroScl, GyroSda>,
) -> Result<(), Lsm303Error> {
    magnetometer.setup().await?;
    loop {
        debounced_press(but).await;
        match magnetometer.get_orientation().await {
            Err(Lsm303Error::Communication(x)) => {
                println!("Error {:?}", x as usize);
                panic!("I2C Error");
            }
            Err(Lsm303Error::General) => println!("General"),
            Ok((x, y, z)) => println! {" x: {} y: {} z: {}",x,y,z},
        }
        but.wait_for_release().await;
    }
}

pub async fn task_blinky(mut led: EastLed) -> Infallible {
    loop {
        led.toggle();
        lilos::exec::sleep_for(Duration::from_millis(1000)).await;
    }
}

pub async fn wake() -> Infallible {
    loop {
        wake_tasks_by_mask(usize::MAX);
        yield_cpu().await
    }
}
