pub trait Led {
    fn on(&mut self);
    fn off(&mut self);
    fn toggle(&mut self);
}
pub mod simple_led;
