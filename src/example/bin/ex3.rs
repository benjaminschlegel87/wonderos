#![no_main]
#![no_std]
use core::convert::Infallible;
use core::future::Future;
use core::panic::PanicInfo;
use core::pin::pin;
use core::pin::Pin;
use defmt::println;
use defmt_rtt as _;

use stm32f3xx_hal as _;
#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
// join executor ???
// first Ready breaks loop
fn execute<T>(task: &mut [Pin<&mut dyn Future<Output = T>>]) {
    let w = lilos::exec::noop_waker();
    let mut c = core::task::Context::from_waker(&w);
    loop {
        for t in task.iter_mut() {
            match t.as_mut().poll(&mut c) {
                core::task::Poll::Ready(_) => break,
                core::task::Poll::Pending => println!("Pending in execute"),
            }
        }
    }
}
#[cortex_m_rt::entry]
fn main() -> ! {
    {
        let mut f = task_statemachine();
        println!("got future");
        let mut pf = pin!(f);
        println!("pinned it");
        let w = lilos::exec::noop_waker();
        let mut c = core::task::Context::from_waker(&w);

        println!("before poll");
        let _ = pf.as_mut().poll(&mut c);
        println!("ran all code up to await point + poll impl once");
        let _ = pf.as_mut().poll(&mut c);
        println!("ran only poll");
        let _ = pf.as_mut().poll(&mut c);
        println!(
            "now i started with poll impl => Ready therefore I ran all code to next await point"
        );
    }
    {
        println!("next future");
        let mut f = standalone_task();
        let mut pf = pin!(f);
        let w = lilos::exec::noop_waker();
        let mut c = core::task::Context::from_waker(&w);

        loop {
            if pf.as_mut().poll(&mut c) == core::task::Poll::Ready(()) {
                break;
            }
        }
        println!("Done");
    }

    {
        println!("next future");
        let mut f = standalone_task();
        let mut f2 = standalone_task();
        let pf = pin!(f);
        let pf2 = pin!(f2);
        execute(&mut [pf, pf2]);
    }

    println!("End");
    loop {}
}

async fn standalone_task() {
    StateA(0, 5).await;
}

struct StateA(usize, usize);

// Resolves after X calls to poll
impl Future for StateA {
    type Output = ();

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if self.0 >= self.1 {
            println!("polled ready");
            core::task::Poll::Ready(())
        } else {
            self.0 += 1;
            println!("pending");
            cx.waker().wake_by_ref();
            core::task::Poll::Pending
        }
    }
}

async fn task_statemachine() -> Infallible {
    loop {
        println!("In Statemachine");
        StateA(0, 2).await;
        println!("After first wait");
        StateA(0, 5).await;
    }
}
