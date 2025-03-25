use std::sync::Arc;

use tokio::{sync::{watch::{self}, Mutex, RwLock}, time::{self, Duration}};

use thiserror::Error;

use crate::countdown::Result;

use super::{CountdownError, Receiver, Response, Sender};

const DEFAULT_TIMEOUT_MS: u32 = 1000;
const DEFAULT_PERIOD_MS: u16 = 100;
const DEFAULT_ACK_POLL_MS: u8 = 10;

type ChanResult<T> = std::result::Result<T, ChannelError>;

trait AwaitWithTimeout<T> {
    fn await_with_timeout(&mut self, timeout: Duration, retry_period: Duration) -> impl Future<Output = ChanResult<T>>;
}

#[derive(Debug, Error, PartialEq)]
pub enum ChannelError {
    #[error("timed out after {0:?} waiting for update")] 
    Timeout(Duration),
}

#[derive(Debug)]
pub(super) struct Channel<T: Copy> {
    tx: Arc<Mutex<watch::Sender<T>>>,
    rx: Arc<Mutex<watch::Receiver<T>>>,
    ack_tx: Arc<Mutex<watch::Sender<bool>>>,
    ack_rx: Arc<Mutex<watch::Receiver<bool>>>,

    closed: Arc<RwLock<bool>>,

    timeout_ms: u32,
    retry_period_ms: u16,
    ack_poll_ms: u8,
}

#[derive(Debug)]
pub struct ChannelSender<T: Copy>(Arc<Channel<T>>);

#[derive(Debug)]
pub struct ChannelReceiver<T: Copy>(Arc<Channel<T>>);

type Mutator<T> = Box<dyn FnOnce(&mut T)>;

pub fn with_timeout<T: Copy>(timeout_ms: u32) -> Mutator<Channel<T>> {
    Box::new(move |watcher| {
        watcher.timeout_ms = timeout_ms;
    })
}

pub fn with_retry_period<T: Copy>(period_ms: u16) -> Mutator<Channel<T>> {
    Box::new(move |watcher| {
        watcher.retry_period_ms = period_ms;
    })
}

impl<T: Copy + PartialEq> Channel<T> {
    pub fn new(init: T) -> (ChannelSender<T>, ChannelReceiver<T>) {
        Self::new_with_options(init, [])
    }

    pub fn new_with_options(init: T, mutators: impl IntoIterator<Item = Mutator<Channel<T>>>) -> (ChannelSender<T>, ChannelReceiver<T>) {
        let (tx, mut rx) = watch::channel(init);
        let (ack_tx, ack_rx) = watch::channel(false);
        rx.mark_changed();

        let mut channel = Channel { 
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),

            ack_tx: Arc::new(Mutex::new(ack_tx)),
            ack_rx: Arc::new(Mutex::new(ack_rx)),

            closed: Arc::new(RwLock::new(false)),

            timeout_ms: DEFAULT_TIMEOUT_MS,
            retry_period_ms: DEFAULT_PERIOD_MS,
            ack_poll_ms: DEFAULT_ACK_POLL_MS,
        };

        mutators.into_iter().for_each(|mutator| mutator(&mut channel));

        let chan = Arc::new(channel);
        (ChannelSender(chan.clone()), ChannelReceiver(chan))
    }

    async fn read(&self) -> ChanResult<Response<T>> {
        if self.closed.read().await.clone() {
            return Ok(Response::Closed);
        }

        let val = self.rx.lock().await.await_with_timeout(
            Duration::from_millis(self.timeout_ms.into()), 
            Duration::from_millis(self.retry_period_ms.into())
        ).await?;

        Ok(Response::Value(val))
    }

    async fn ack(&self) -> ChanResult<()> {
        // TODO: Add timeout
        self.ack_tx.lock().await.send_replace(true);

        Ok(())
    }

    async fn write(&self, value: T) -> Result<()> {
        let tx = self.tx.lock().await;
        tx.send_modify(|v| *v = value);
        
        Ok(())
    }

    async fn wait_ack(&self) -> ChanResult<()> {
        self.ack_rx.lock().await.await_with_timeout(
            Duration::from_millis(self.timeout_ms.into()), 
            Duration::from_millis(self.ack_poll_ms.into())
        ).await?;

        self.ack_tx.lock().await.send_replace(false);

        Ok(())
    }
}

impl<T: Copy + PartialEq> Receiver<T> for ChannelReceiver<T> {
    async fn recv(&self) -> Result<super::Response<T>> {
        let chan = self.0.clone();

        let response = chan.read().await.map_err(CountdownError::from)?;
        chan.ack().await.map_err(CountdownError::from)?;

        Ok(response)
    }
}

impl<T: Copy + PartialEq> Sender<T> for ChannelSender<T> {
    async fn send(&self, value: T) -> Result<()> {
        // TODO: Add a timeout
        let chan = self.0.clone();
        
        chan.write(value).await.map_err(CountdownError::from)
    }

    async fn close(&self) -> Result<()> {
        // TODO: Add a timeout
        let chan = self.0.clone();

        chan.wait_ack().await.map_err(CountdownError::from)?;
        *chan.closed.write().await = true;

        Ok(())
    }
}

impl<T: Clone> AwaitWithTimeout<T> for watch::Receiver<T> {
    async fn await_with_timeout(&mut self, timeout: Duration, retry_period: Duration) -> ChanResult<T> {
        let poll = async |rx: &mut watch::Receiver<T>| {
            let val_ref = rx.borrow_and_update();

            if !val_ref.has_changed() {
                return None;
            }

            Some(val_ref.clone())
        };

        let wait_for_changed_value = async {
            loop {
                if let Some(v) = poll(self).await {
                    return v;
                }

                time::sleep(retry_period).await;
            }   
        };

        time::timeout(timeout, wait_for_changed_value).await
            .map_err(|_| ChannelError::Timeout(timeout))
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::Duration;

    use crate::countdown::{CountdownError, Response};

    use super::*;

    #[tokio::test]
    async fn should_timeout_if_no_updates_received_for_more_than_the_default_timeout_of_1_second() {
        time::pause();
        let (_, rx) = Channel::new(42u32);
        assert_eq!(rx.recv().await.expect("unexpected error"), Response::Value(42));
        
        time::advance(Duration::from_millis(1001)).await;
        assert_eq!(rx.recv().await.expect_err("expected error"), CountdownError::ChannelError(ChannelError::Timeout(Duration::from_millis(1000))));
    }

    #[tokio::test]
    async fn should_timeout_if_no_updates_received_for_more_than_the_specified_timeout() {
        time::pause();
        let (_, rx) = Channel::new_with_options(42u32, [with_timeout(500)]);
        assert_eq!(rx.recv().await.expect("unexpected error"), Response::Value(42));
        
        time::advance(Duration::from_millis(501)).await;
        assert_eq!(rx.recv().await.expect_err("expected error"), CountdownError::ChannelError(ChannelError::Timeout(Duration::from_millis(500))));
    }

    #[tokio::test]
    async fn should_return_the_initial_value() {
        let (_, rx) = Channel::new(42u32);
     
        assert_eq!(rx.recv().await.expect("unexpected error"), Response::Value(42));
    }

    #[tokio::test]
    async fn should_return_closed_when_the_sender_is_closed() {
        let (tx, rx) = Channel::new(0u32);
        
        assert_eq!(rx.recv().await.expect("unexpected error awaiting initial value"), Response::Value(0));

        tx.close().await.expect("unexpected error closing channel");

        assert_eq!(rx.recv().await.expect("unexpected error awaiting closed"), Response::Closed);
    }

    #[tokio::test]
    async fn should_return_some_if_value_is_not_zero() {
        let (tx, rx) = Channel::new(100u32);

        assert_eq!(rx.recv().await.expect("unexpected error awaiting initial value"), Response::Value(100));
        tx.send(50).await.expect("unexpected error sending value");

        assert_eq!(rx.recv().await.expect("unexpected error awaiting updated value"), Response::Value(50));
    }

    #[tokio::test]
    async fn should_return_latest_value_only() {
        let (tx, rx) = Channel::new(100u32);

        assert_eq!(rx.recv().await.expect("unexpected error awaiting initial value"), Response::Value(100));
        tx.send(99).await.expect("unexpected error sending value");
        tx.send(50).await.expect("unexpected error sending value");

        assert_eq!(rx.recv().await.expect("unexpected error awaiting updated value"), Response::Value(50));
 
        tx.send(25).await.expect("unexpected error sending value");
        tx.close().await.expect("unexpected error closing channel");

        assert_eq!(rx.recv().await.expect("unexpected error awaiting closed"), Response::Closed);
    }

    #[tokio::test]
    async fn should_wait_for_ack_before_closing() {
        let (tx, rx) = Channel::new(0u32);

        let tx_handle = tokio::spawn(async move { tx.close().await.expect("unexpected error closing channel") });
        let rx_handle = tokio::spawn(async move { 
            assert_eq!(rx.recv().await.expect("unexpected error awaiting initial value"), Response::Value(0));
            assert_eq!(rx.recv().await.expect("unexpected error awaiting closed"), Response::Closed);
        });

        tokio::select! {
            res = tx_handle => { res.unwrap() }
            res = rx_handle => { res.unwrap() }
        };
    }
}
