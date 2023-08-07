#![no_std]
pub mod async_spi;
pub mod button;
pub mod led;
pub mod stm32f3_disco_def;
use defmt::println;

pub async fn debounced_press(but: &stm32f3_disco_def::UserButton) -> bool {
    but.wait_for_press().await;
    match lilos::exec::with_timeout(
        core::time::Duration::from_millis(100),
        but.wait_for_release(),
    )
    .await
    {
        None => {
            println!("Valid Press");
            return true;
        }
        Some(_res) => {
            println!("Timeout: No valid Press");
            return false;
        } //
    }
}
