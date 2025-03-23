use std::sync::Arc;

use tokio::{sync::RwLock, time::{self, Duration}};

use thiserror::Error;
use tokio::sync::watch::Receiver;

use super::Watcher;
use crate::countdown::Result;

const DEFAULT_TIMEOUT_MS: u32 = 1000;

pub trait Zeroable: Copy + PartialEq + Eq {
    fn is_zero(&self) -> bool;
}

#[derive(Debug, Error, PartialEq)]
pub enum WatcherError {
    #[error("EOF")]
    EOF,
    #[error("timed out after {0:?} waiting for update")] 
    Timeout(Duration),
}

#[derive(Debug)]
pub struct ChannelWatcher<T> {
    timeout_ms: u32,
    rx: Arc<RwLock<Receiver<T>>>,
}

macro_rules! channel {
    ($rx:expr) => {
        ChannelWatcher {
            rx: Arc::new(RwLock::new($rx)),
            timeout_ms: DEFAULT_TIMEOUT_MS
        }
    };
    ($rx:expr, $timeout_ms:expr) => {
        ChannelWatcher {
            rx: Arc::new(RwLock::new($rx)),
            timeout_ms: $timeout_ms
        }
    };
}


impl<T: Zeroable + Copy> Watcher<T> for ChannelWatcher<T> {
    async fn next(&mut self) -> Result<Option<T>> {
        let timeout = Duration::from_millis(self.timeout_ms.into());
        time::timeout(timeout, async {self.rx.write().await.changed().await.unwrap()}).await.or(Err(WatcherError::Timeout(timeout)))?;

        let val = self.rx.read().await.borrow().clone();
        if val.is_zero() {
            return Ok(None);
        }

        Ok(Some(val))
    }
}

impl<T: Zeroable + Copy> From<Receiver<T>> for ChannelWatcher<T> {
    fn from(rx: Receiver<T>) -> Self {
        channel!(rx)
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
    use tokio::{sync::watch, time::{self, Duration}};

    use super::*;
    use crate::countdown::Watcher;

    #[tokio::test]
    async fn should_timeout_if_no_updates_received_for_more_than_1_second() {
        time::pause();
        let (_tx, rx) = watch::channel(42u32);
        
        time::advance(Duration::from_millis(1001)).await;
        assert_eq!(channel!(rx).next().await.expect_err("expected error"), WatcherError::Timeout(Duration::from_millis(1000)).into());
    }

    #[tokio::test]
    async fn should_return_none_if_value_is_zero() {
        let (tx, rx) = watch::channel(42u32);
        tx.send(0).unwrap();

        assert_eq!(channel!(rx).next().await.expect("unexpected error"), None);
    }

    #[tokio::test]
    async fn should_return_some_if_value_is_not_zero() {
        let (tx, rx) = watch::channel(0u32);
        tx.send(42).unwrap();

        assert_eq!(channel!(rx).next().await.expect("unexpected error"), Some(42));
    }
}
