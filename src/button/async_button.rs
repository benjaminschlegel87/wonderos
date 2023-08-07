use core::future::Future;
use core::task::Poll;

use super::simple_button::{InputPin, SimpleButton};

pub enum ButtonAction {
    WaitForRelease,
    WaitForPress,
}

pub struct AsyncButton<'a, T: InputPin> {
    button_action: ButtonAction,
    but: &'a SimpleButton<T>,
}
impl<'a, T: InputPin> AsyncButton<'a, T> {
    pub fn new(but: &'a SimpleButton<T>, button_action: ButtonAction) -> Self {
        Self { button_action, but }
    }
}
impl<T: InputPin> SimpleButton<T> {
    pub async fn wait_for_press(&self) {
        AsyncButton::new(&self, ButtonAction::WaitForPress).await;
    }
    pub async fn wait_for_release(&self) {
        AsyncButton::new(&self, ButtonAction::WaitForRelease).await;
    }
}
impl<'a, T: InputPin> Future for AsyncButton<'a, T> {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        match self.button_action {
            ButtonAction::WaitForPress => {
                if self.but.is_pressed() == true {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
            ButtonAction::WaitForRelease => {
                // We check for a release
                if self.but.is_pressed() == false {
                    Poll::Ready(())
                } else {
                    //_cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        }
    }
}
