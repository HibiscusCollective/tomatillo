use tokio::sync::watch::Receiver;

use super::Watcher;

pub trait Zeroable: Copy + PartialEq + Eq {
    fn is_zero(&self) -> bool;
}

#[derive(Debug)]
pub struct ChannelWatcher<T> {
    rx: Receiver<T>,
}

impl<T: Zeroable + Copy> Watcher<T> for ChannelWatcher<T> {
    async fn next(&mut self) -> Option<T> {
        self.rx.changed().await.expect("sender was unexpectedly dropped");

        let val = self.rx.borrow_and_update();
        if val.is_zero() {
            return None;
        }

        Some(val.clone())
    }
}

impl<T: Zeroable + Copy> From<Receiver<T>> for ChannelWatcher<T> {
    fn from(rx: Receiver<T>) -> Self {
        Self { rx }
    }
}

macro_rules! impl_zeroable {
    ($($t:ty),*) => {
        $(
            impl Zeroable for $t {
                fn is_zero(&self) -> bool {
                    *self == 0
                }
            }
        )*
    }
}

impl_zeroable!(u8, u16, u32, u64, u128);

#[cfg(test)]
mod tests {
    use tokio::sync::watch;

    use super::*;

    #[tokio::test]
    async fn should_return_none_if_value_is_zero() {
        let (tx, rx) = watch::channel(42u32);
        tx.send(0).unwrap();

        let mut watcher = ChannelWatcher::from(rx);
        assert_eq!(watcher.next().await, None);
    }

    #[tokio::test]
    async fn should_return_some_if_value_is_not_zero() {
        let (tx, rx) = watch::channel(0u32);
        tx.send(42).unwrap();

        let mut watcher = ChannelWatcher::from(rx);
        assert_eq!(watcher.next().await, Some(42));
    }
}
