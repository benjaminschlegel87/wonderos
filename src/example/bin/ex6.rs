#![no_main]
#![no_std]
use core::panic::PanicInfo;
use core::pin::pin;
use defmt::println;
use defmt_rtt as _;
use stm32f3xx_hal::pac::CorePeripherals;
use stm32f3xx_hal::pac::Peripherals;
use wonderos::stm32f3_disco_def::Board;
#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    println!("Paniced");
    loop {}
}
#[cortex_m_rt::entry]
fn main() -> ! {
    // Take Cortex-M and STM32 Peripherials
    let mut core = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();

    // Create Board Abstraction
    let b = Board::new(p);

    // Basic Blinky Task
    let t = pin!(wonderos::task_blinky(b.east_led,));
    // Always Wake all Tasks Task
    let w = pin!(wonderos::wake());
    // Magnetometer logic task
    let g = pin!(wonderos::task_magnetometer(b.user_button, b.magnetometer));
    // Give lilos a systick to provide delays
    lilos::time::initialize_sys_tick(&mut core.SYST, b.clocks.sysclk().0);
    // Run tasks forever
    lilos::exec::run_tasks(&mut [t, g, w], lilos::exec::ALL_TASKS);
}
