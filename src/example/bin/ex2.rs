#![no_main]
#![no_std]
use core::convert::Infallible;
use core::future::Future;
use core::panic::PanicInfo;
use core::pin::pin;
use core::time::Duration;
use defmt::println;
use defmt_rtt as _;
use lilos::exec::Notify;
use stm32f3xx_hal::pac::CorePeripherals;
use stm32f3xx_hal::pac::Peripherals;
use wonderos::led::Led;
use wonderos::stm32f3_disco_def::UserButton;
use wonderos::stm32f3_disco_def::{Board, EastLed, SouthEastLed, SouthLed};
#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}

// Notify pattern erkunden => Split Button Erkennung und Led Toggle + andere Aktion in verschiedene Tasks mit Notify Wakern
// Man muss entweder in jeder Future Impl den Waker (Muss man wohl immer)
// Oder man muss ein Notify Ref in jedem Future übergeben damit man den Waker Subscriben sind

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut core = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();

    let mut b = Board::new(p);
    let n = Notify::new();
    let blinky = pin!(task_blinky(&mut b.east_led));
    let button = pin!(task_button_pressed(&b.user_button, &n));
    let resp = pin!(task_toggle_on_press(&mut b.southeast_led, &n));
    let wake = pin!(wake_all());

    lilos::time::initialize_sys_tick(&mut core.SYST, 8_000_000);
    lilos::exec::run_tasks(&mut [blinky, button, resp, wake], lilos::exec::ALL_TASKS);
}

async fn task_blinky(led: &mut EastLed) -> Infallible {
    loop {
        led.toggle();
        lilos::exec::sleep_for(Duration::from_millis(1000)).await;
    }
}

async fn task_button_pressed(but: &UserButton, notify: &lilos::exec::Notify) -> Infallible {
    loop {
        if wonderos::debounced_press(but).await {
            notify.notify();
            but.wait_for_release().await;
        }
    }
}
use core::cell::Cell;
struct WaitForBut<'a> {
    notify: &'a Notify,
    first: Cell<bool>,
}
impl<'a> WaitForBut<'a> {
    pub fn new(notify: &'a Notify) -> Self {
        Self {
            notify,
            first: Cell::new(false),
        }
    }
}
impl<'a> Future for WaitForBut<'a> {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if self.first.replace(true) == false {
            self.notify.subscribe(cx.waker());
            core::task::Poll::Pending
        } else {
            core::task::Poll::Ready(())
        }
    }
}

async fn task_toggle_on_press(led: &mut SouthEastLed, not: &Notify) -> Infallible {
    loop {
        WaitForBut::new(not).await;
        led.toggle();
    }
}

async fn wake_all() -> Infallible {
    loop {
        // wake only Task 1
        // wake all tasks
        lilos::exec::wake_tasks_by_mask(1 << 1 as usize);
        lilos::exec::yield_cpu().await;
    }
}
