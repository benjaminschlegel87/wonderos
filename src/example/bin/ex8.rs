#![no_main]
#![no_std]
use core::panic::PanicInfo;
use core::sync::atomic::AtomicUsize;
use cortex_m_rt::exception;
use defmt_rtt as _;
use stm32f3xx_hal as _;
use stm32f3xx_hal::pac::{CorePeripherals, Peripherals};

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
static CNT: AtomicUsize = AtomicUsize::new(0);
#[cortex_m_rt::entry]
fn main() -> ! {
    let p = Peripherals::take().unwrap();
    p.RCC.ahbenr.modify(|_, w| w.iopeen().enabled());
    p.GPIOE.moder.modify(|_, w| w.moder10().output());
    p.GPIOE.bsrr.write(|w| w.bs10().set());

    let mut core = CorePeripherals::take().unwrap();
    core.SYST.set_reload(2_000);
    core.SYST.enable_counter();
    core.SYST.enable_interrupt();
    loop {
        if CNT
            .compare_exchange(
                1000,
                0,
                core::sync::atomic::Ordering::Relaxed,
                core::sync::atomic::Ordering::Relaxed,
            )
            .is_ok()
        {
            if p.GPIOE.odr.read().odr10().is_high() {
                p.GPIOE.bsrr.write(|w| w.br10().set_bit());
            } else {
                p.GPIOE.bsrr.write(|w| w.bs10().set());
            }
        }
    }
}

#[exception]
fn SysTick() {
    CNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
}
