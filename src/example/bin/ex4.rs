#![no_main]
#![no_std]
use core::convert::Infallible;
use core::future::Future;
use core::panic::PanicInfo;
use core::pin::pin;
use core::task::Poll;
use core::time::Duration;
use defmt::println;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use nb::block;
use stm32f3xx_hal::pac::{CorePeripherals, Peripherals};
use stm32f3xx_hal::prelude::_embedded_hal_blocking_spi_Transfer;
use stm32f3xx_hal::time::fixed_point::FixedPoint;
use wonderos::stm32f3_disco_def::Board;
#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
struct AsynSpiRead<'a, T: FullDuplex<u8>> {
    spi: &'a mut T,
}

impl<'a, T: FullDuplex<u8>> Future for AsynSpiRead<'a, T> {
    type Output = Result<u8, ()>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let r = self.spi.read();
        match r {
            Ok(res) => Poll::Ready(Ok(res)),
            Err(nb::Error::Other(_)) => Poll::Ready(Err(())),
            Err(nb::Error::WouldBlock) => {
                println!("would block");
                return Poll::Pending;
            }
        }
    }
}

struct AsynSpiWrite<'a, T: FullDuplex<u8>> {
    spi: &'a mut T,
    payload: u8,
}
impl<'a, T: FullDuplex<u8>> Future for AsynSpiWrite<'a, T> {
    type Output = Result<(), ()>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let p = self.payload;
        let r = self.spi.send(p);
        match r {
            Ok(_) => Poll::Ready(Ok(())),
            Err(nb::Error::Other(_)) => Poll::Ready(Err(())),
            Err(nb::Error::WouldBlock) => Poll::Pending,
        }
    }
}
const CMD: u8 = 143; // [Bit 7 = 1 (Read), Bit 6 = 0 (Not increment adr), Bit 5..0 = 0xF (adr who am i)]
#[cortex_m_rt::entry]
fn main() -> ! {
    let mut core = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();
    let mut b = Board::new(p);
    let _ = b.gyro_cs.set_low();
    let mut cmd: u8 = 3 << 7;
    cmd |= 0xF;
    println!("Cmd {}", cmd);
    block!(b.gyro_spi.send(cmd)).unwrap();
    let res = block!(b.gyro_spi.read()).unwrap();
    println!("Res {}", res);
    block!(b.gyro_spi.send(0 as u8)).unwrap();
    let res = block!(b.gyro_spi.read()).unwrap();
    println!("Res {}", res);

    let _ = b.gyro_cs.set_high();

    let _ = b.gyro_cs.set_low();
    let mut payload = [cmd, 0];
    let r = b.gyro_spi.transfer(&mut payload).unwrap();
    println!(" other res {}", r);

    let _ = b.gyro_cs.set_high();
    let read = pin!(read_who_am_i_async(&mut b.gyro_spi, &mut b.gyro_cs));
    let wake = pin!(wake());

    lilos::time::initialize_sys_tick(&mut core.SYST, b.clocks.sysclk().integer());
    lilos::exec::run_tasks(&mut [read, wake], lilos::exec::ALL_TASKS);
}

async fn read_who_am_i_async(
    spi: &mut wonderos::stm32f3_disco_def::GyroSpi,
    cs: &mut wonderos::stm32f3_disco_def::GyroCs,
) -> Infallible {
    loop {
        cs.set_low().expect("");
        AsynSpiWrite { spi, payload: CMD }.await.unwrap();
        println!("Res {}", AsynSpiRead { spi }.await.unwrap());
        AsynSpiWrite {
            spi,
            payload: 0 as u8,
        }
        .await
        .unwrap();
        println!("Res {}", AsynSpiRead { spi }.await.unwrap());
        cs.set_high().expect("");
        lilos::exec::sleep_for(Duration::from_millis(1000)).await
    }
}

async fn wake() -> Infallible {
    loop {
        lilos::exec::wake_tasks_by_mask(usize::MAX);
        lilos::exec::yield_cpu().await
    }
}
