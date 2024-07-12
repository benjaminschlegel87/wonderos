#![no_main]
#![no_std]
use core::mem::size_of;
use core::panic::PanicInfo;
//
use defmt_rtt as _;
// stm32f3xx_hal provides a memory.x linker file which is nedded by cortex-m-rt
// we could put a memory.x in this crate directly so we would not need this line
use stm32f3xx_hal as _;
#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}

const RCC_BASE: *const u32 = 0x4002_1000 as *const _;
const RCC_AHBENR: usize = 0x14;

const GPIOE_BASE: *const u32 = 0x4800_1000 as *const _;
const GPIO_MODER_OFFSET: usize = 0x00;
const _GPIO_OTYPER_OFFSET: usize = 0x04;
const _GPIO_OSPEEDR_OFFSET: usize = 0x08;
const _GPIO_PUPDR_OFFSET: usize = 0x0c;
const _GPIO_IDR_OFFSET: usize = 0x10;
const _GPIO_ODR_OFFSET: usize = 0x14;
const GPIO_BSSR_OFFSET: usize = 0x18;
#[cortex_m_rt::entry]
fn main() -> ! {
    // Create Pointer to AHBENR Register
    let ahbenr = unsafe { RCC_BASE.add(RCC_AHBENR / size_of::<usize>()).cast_mut() };
    // Read Register
    let mut reg = unsafe { *ahbenr };
    // GPIOE EN Bit 21
    reg |= 1 << 21;
    // Write new value to Register
    unsafe {
        ahbenr.write_volatile(reg);
    }
    // Create Pointer to MODER Register
    let moder = unsafe {
        // b01 => output
        GPIOE_BASE
            .add(GPIO_MODER_OFFSET / size_of::<usize>())
            .cast_mut()
    };

    // Read Register
    let mut reg = unsafe { *moder };
    // 0b01 => Output Pin 10 => 20 Shifts
    reg |= 0b01 << 20;
    // Write new value to Register
    unsafe {
        moder.write_volatile(reg);
    }

    // BSSR Register no need to Read back
    let bssr = unsafe {
        // Create Pointer to GPIOE BSSR
        GPIOE_BASE
            .add(GPIO_BSSR_OFFSET / size_of::<usize>())
            .cast_mut()
    };

    unsafe {
        // Write Set Bit Pin 10
        bssr.write_volatile(1 << 10);
    }

    run(bssr);
}
pub fn run(bssr: *mut u32) -> ! {
    loop {
        // ~1s Blink pattern
        for _i in 0..1_000_000 {
            cortex_m::asm::nop();
        }

        unsafe {
            // Write Set Bit Pin 10
            bssr.write_volatile(1 << 26);
        }
        for _i in 0..1000000 {
            cortex_m::asm::nop();
        }

        unsafe {
            // Write Set Bit Pin 10
            bssr.write_volatile(1 << 10);
        }
    }
}
