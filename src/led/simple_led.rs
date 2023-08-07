use crate::led::Led;
use embedded_hal::digital::v2::OutputPin;
pub struct SimpleLed<Pin: OutputPin> {
    pin: Pin,
    is_state_high: bool,
}

impl<Pin: OutputPin> SimpleLed<Pin> {
    pub fn new(pin: Pin) -> Self {
        Self {
            pin,
            is_state_high: false,
        }
    }
}
impl<Pin: OutputPin> Led for SimpleLed<Pin> {
    fn on(&mut self) {
        if let Ok(()) = self.pin.set_high() {
        } else {
            // should never happen
            panic!()
        }
    }
    fn off(&mut self) {
        if let Ok(()) = self.pin.set_low() {
        } else {
            // should never happen
            panic!()
        }
    }
    fn toggle(&mut self) {
        if self.is_state_high {
            self.off();
            self.is_state_high = false;
        } else {
            self.on();
            self.is_state_high = true;
        }
    }
}
#[cfg(test)]
pub mod mock_outputpin {
    use core::cell::RefCell;
    use embedded_hal::digital::v2::OutputPin;
    use std::rc::Rc;

    pub struct MockOutputPin {
        shared: Rc<RefCell<Shared>>,
    }
    pub struct SpyOutputPin {
        shared: Rc<RefCell<Shared>>,
    }

    #[derive(Debug, Default)]
    struct Shared {
        low_cnt: usize,
        high_cnt: usize,
        err_on_low: bool,
        err_on_high: bool,
    }
    impl MockOutputPin {
        pub fn new() -> (Self, SpyOutputPin) {
            let spy = SpyOutputPin {
                shared: Rc::new(RefCell::new(Shared::default())),
            };
            (
                MockOutputPin {
                    shared: spy.shared.clone(),
                },
                spy,
            )
        }
    }
    impl OutputPin for MockOutputPin {
        type Error = ();

        fn set_low(&mut self) -> Result<(), Self::Error> {
            let mut inner = self.shared.borrow_mut();
            if inner.err_on_low == true {
                Err(())
            } else {
                inner.low_cnt += 1;
                Ok(())
            }
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            let mut inner = self.shared.borrow_mut();
            if inner.err_on_high == true {
                Err(())
            } else {
                inner.high_cnt += 1;
                Ok(())
            }
        }
    }
    impl SpyOutputPin {
        pub fn reset(&mut self) {
            let mut inner = self.shared.borrow_mut();
            inner.err_on_high = Default::default();
            inner.err_on_low = Default::default();
            inner.low_cnt = Default::default();
            inner.high_cnt = Default::default();
        }
        pub fn get_high_cnt(&self) -> usize {
            self.shared.borrow().high_cnt
        }
        pub fn get_low_cnt(&self) -> usize {
            self.shared.borrow().low_cnt
        }
        pub fn set_low_err(&mut self, is_err: bool) {
            self.shared.borrow_mut().err_on_low = is_err;
        }
        pub fn set_high_err(&mut self, is_err: bool) {
            self.shared.borrow_mut().err_on_high = is_err;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::led::simple_led::mock_outputpin::MockOutputPin;

    use super::*;
    #[test]
    fn basic_usage() {
        // Test of Basic expected usage
        let (pin_mock, pin_spy) = MockOutputPin::new();
        assert_eq!(pin_spy.get_low_cnt(), 0);
        assert_eq!(pin_spy.get_high_cnt(), 0);
        let mut led = SimpleLed::new(pin_mock);
        led.on();
        assert_eq!(pin_spy.get_high_cnt(), 1);
        led.off();
        assert_eq!(pin_spy.get_low_cnt(), 1);
    }

    #[should_panic]
    #[test]
    fn panic_on_error_low() {
        // make sure that we panic on Output Pin Error
        let (pin_mock, mut pin_spy) = MockOutputPin::new();
        let mut led = SimpleLed::new(pin_mock);
        pin_spy.set_low_err(true);
        led.off();
    }

    #[should_panic]
    #[test]
    fn panic_on_error_high() {
        // make sure that we panic on Output Pin Error
        let (pin_mock, mut pin_spy) = MockOutputPin::new();
        let mut led = SimpleLed::new(pin_mock);
        pin_spy.set_high_err(true);
        led.on();
    }
    #[test]
    fn with_toggle() {
        // Test optional toggle feature
        let (pin_mock, pin_spy) = MockOutputPin::new();
        assert_eq!(pin_spy.get_low_cnt(), 0);
        assert_eq!(pin_spy.get_high_cnt(), 0);
        let mut led = SimpleLed::new(pin_mock);
        led.toggle();
        assert_eq!(pin_spy.get_high_cnt(), 1);
        led.toggle();
        assert_eq!(pin_spy.get_low_cnt(), 1);
    }
}
