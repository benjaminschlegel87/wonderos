pub use embedded_hal::digital::v2::InputPin;
/// Defines the polarity logic of the connected switch
#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub enum Polarity {
    /// Input Pin yields High => Button is active<br>
    ActiveHigh,
    /// Input Pin yields low => Button is active
    ActiveLow,
}
/// Struct representing a Button
pub struct SimpleButton<T: InputPin> {
    button_pin: T,
    polarity: Polarity,
}

pub trait IsButton {
    fn is_pressed(&self) -> bool;
}
impl<T: InputPin> SimpleButton<T> {
    /// Creates a new Button from given [InputPin] and the [Polarity]
    pub fn new(pin: T, polarity: Polarity) -> Self {
        Self {
            button_pin: pin,
            polarity,
        }
    }
    pub fn get_polarity(&self) -> Polarity {
        self.polarity
    }
    /// Returns if the button is pressed
    pub fn is_pressed(&self) -> bool {
        let is_low = if let Ok(is_low) = self.button_pin.is_low() {
            is_low
        } else {
            // should never happen
            panic!()
        };
        match self.polarity {
            Polarity::ActiveLow => {
                if is_low {
                    true
                } else {
                    false
                }
            }
            Polarity::ActiveHigh => {
                if is_low {
                    false
                } else {
                    true
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    mod mocks {
        use core::cell::RefCell;
        use embedded_hal::digital::v2::InputPin;
        use std::rc::Rc;
        #[derive(Clone)]
        pub struct InputPinMock {
            shared: Rc<RefCell<Shared>>,
        }
        pub struct InputPinSpy {
            shared: Rc<RefCell<Shared>>,
        }
        struct Shared {
            pin_is_low: bool,
            ret_err: bool,
        }
        impl InputPinMock {
            pub fn new() -> (Self, InputPinSpy) {
                let mock = Self {
                    shared: Rc::new(RefCell::new(Shared {
                        pin_is_low: false,
                        ret_err: false,
                    })),
                };
                let spy = InputPinSpy {
                    shared: mock.shared.clone(),
                };
                (mock, spy)
            }
        }
        impl InputPin for InputPinMock {
            type Error = ();

            fn is_high(&self) -> Result<bool, Self::Error> {
                let shared = self.shared.as_ref().borrow();
                if true == shared.ret_err {
                    Err(())
                } else {
                    if true == shared.pin_is_low {
                        Ok(false)
                    } else {
                        Ok(true)
                    }
                }
            }

            fn is_low(&self) -> Result<bool, Self::Error> {
                let shared = self.shared.as_ref().borrow();
                if true == shared.ret_err {
                    Err(())
                } else {
                    if true == shared.pin_is_low {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                }
            }
        }
        impl InputPinSpy {
            pub fn set_result(&mut self, is_low: bool) {
                self.shared.as_ref().borrow_mut().pin_is_low = is_low;
            }
            pub fn set_err(&mut self, is_err: bool) {
                self.shared.as_ref().borrow_mut().ret_err = is_err;
            }
        }
    }

    use self::mocks::InputPinMock;

    use super::*;
    #[test]
    fn basic_usage() {
        let (mock, mut spy) = InputPinMock::new();
        let button = SimpleButton::new(mock, Polarity::ActiveHigh);
        spy.set_result(true);
        assert_eq!(button.is_pressed(), false);

        spy.set_result(false);
        assert_eq!(button.is_pressed(), true);
    }
    #[should_panic]
    #[test]
    fn on_input_err() {
        let (mock, mut spy) = InputPinMock::new();
        let button = SimpleButton::new(mock, Polarity::ActiveHigh);
        spy.set_err(true);
        button.is_pressed();
    }
}
