#![no_main]
#![no_std]
use core::convert::Infallible;
use core::panic::PanicInfo;
use core::pin::pin;
use core::time::Duration;
use defmt::println;
use defmt_rtt as _;
use lilos::exec::{wake_tasks_by_mask, yield_cpu, PeriodicGate};
use stm32f3xx_hal::pac::CorePeripherals;
use stm32f3xx_hal::pac::Peripherals;
use wonderos::led::Led;
use wonderos::stm32f3_disco_def::SouthEastLed;
use wonderos::stm32f3_disco_def::SouthLed;
use wonderos::stm32f3_disco_def::{Board, EastLed, NorthEastLed, UserButton};
#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut core = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();

    let mut b = Board::new(p);

    let t = pin!(task_blinky(&mut b.east_led));
    let t2 = pin!(task_blinky_timegate(&mut b.northeast_led));
    let e = pin!(task_button(b.user_button, &mut b.southeast_led));
    let d = pin!(cont_worker(&mut b.south_led));
    let w = pin!(wake_all());
    lilos::time::initialize_sys_tick(&mut core.SYST, 8_000_000);
    lilos::exec::run_tasks(&mut [t, t2, e, d, w], lilos::exec::ALL_TASKS);
}

async fn task_blinky(led: &mut EastLed) -> Infallible {
    loop {
        led.toggle();
        lilos::exec::sleep_for(Duration::from_millis(1000)).await;
    }
}
async fn task_blinky_timegate(led: &mut NorthEastLed) -> Infallible {
    lilos::exec::sleep_for(Duration::from_millis(500)).await;
    let mut gate = PeriodicGate::from(Duration::from_millis(1000));
    loop {
        gate.next_time().await;
        led.toggle();
    }
}

async fn task_button(but: UserButton, led: &mut SouthEastLed) -> Infallible {
    if but.is_pressed() {
        led.on();
    } else {
        led.off();
    }
    loop {
        if wonderos::debounced_press(&but).await {
            led.toggle();
            but.wait_for_release().await;
        }
    }
}

/// Example of a async function that does not really waits on something put runs as often as it can but gives up
/// control every loop
async fn cont_worker(led: &mut SouthLed) -> Infallible {
    let mut cnt = 0;
    const HIGH: i32 = 10000;
    println!("in worker");
    loop {
        cnt += 1;
        if cnt == HIGH {
            cnt = 0;
            led.toggle();
        }
        yield_cpu().await;
    }
}
/// No need for Waker or Notify needed with this - Continious polling
async fn wake_all() -> Infallible {
    loop {
        // wake only Task 1
        wake_tasks_by_mask(1 << 2 as usize);
        // wake all tasks
        //wake_tasks_by_mask(usize::MAX);
        yield_cpu().await;
    }
}
