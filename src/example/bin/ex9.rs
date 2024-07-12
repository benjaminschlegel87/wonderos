#![no_main]
#![no_std]
use core::panic::PanicInfo;
use defmt_rtt as _;
use embedded_hal::digital::v2::outputpin;
use embedded_hal::timer::CountDown;
use stm32f3xx_hal as _;
use stm32f3xx_hal::pac::Peripherals;
use stm32f3xx_hal::prelude::_stm32f3xx_hal_gpio_GpioExt;
use stm32f3xx_hal::rcc::RccExt;
use stm32f3xx_hal::timer;

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
#[cortex_m_rt::entry]
fn main() -> ! {
    let p = Peripherals::take().unwrap();
    let mut rcc = p.RCC.constrain();
    let gpioe = p.GPIOE;
    let mut pins = gpioe.split(&mut rcc.ahb);
    let pe10 = pins.pe10;
    let mut pe10_output = pe10.into_push_pull_output(&mut pins.moder, &mut pins.otyper);
    pe10_output.set_high().unwrap_or_default();

    let timer = timer::Timer::new(p.TIM1, rcc.freeze(), &mut p.TIM1.arr);
    loop {}
}
