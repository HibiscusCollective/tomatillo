use thiserror::Error;

use crate::countdown::timer::TimerError;

mod timer;
mod channel;

pub use timer::AsyncCountdown;
pub use channel::{ChannelReceiver, ChannelError};

pub type Result<T> = std::result::Result<T, CountdownError>;

#[derive(Debug, Error, PartialEq)]
pub enum CountdownError {
    #[error(transparent)]
    TimerError(#[from] TimerError),
    #[error(transparent)]
    ChannelError(#[from] ChannelError),
}

#[derive(Debug, PartialEq)]
pub enum Response<T: PartialEq + Copy> {
    Value(T),
    Closed,
}

/// A countdown that counts down from a specified duration.
pub trait Countdown<T: Copy> {
    /// Starts the countdown.
    ///
    /// # Arguments
    ///
    /// * `duration_millis` - The duration of the countdown in milliseconds.
    ///
    /// # Returns
    ///
    /// A [`Result`] that is:
    ///
    /// * `Ok(watcher)` - The countdown has started, and a [`ChannelReceiver`] is returned.
    /// * `Err(err)` - The countdown could not be started.
    fn start(&self, duration_millis: u64) -> impl std::future::Future<Output = Result<ChannelReceiver<u64>>>;
}


/// A sender that sends updates countdown updates to a receiver and waits for ack between sends
pub trait Sender<T> {
    /// Sends a value to the [`Receiver`] and waits for the [`Receiver`] to acknowledge receipt.
    /// 
    /// # Arguments
    /// 
    /// * `value` - The value to send to the receiver
    /// 
    /// # Returns
    /// 
    /// A [`Result`] that is:
    /// 
    /// * `Ok(())` - The value has been sent successfully.
    /// * `Err(err)` - The value could not be sent.
    fn send(&self, value: T) -> impl std::future::Future<Output = Result<()>>;

    /// Closes the sender, indicating that no more values will be sent. 
    /// 
    /// Implementations of this function should alert the receiver that the sender is closed to indicate no more 
    /// values will be sent.
    /// 
    /// # Returns
    /// 
    /// A [`Result`] that is:
    /// 
    /// * `Ok(())` - The sender has been closed successfully.
    /// * `Err(err)` - The sender could not be closed.
    fn close(&self) -> impl std::future::Future<Output = Result<()>>;
}

/// Receives updates from a sender and acknowledges receipt
pub trait Receiver<T: PartialEq + Copy> {
    /// Receives a value from the sender and acknowledges receipt
    /// 
    /// # Returns
    /// 
    /// A [`Result`] that is:
    /// 
    /// * `Ok(value)` - The value has been received successfully.
    /// * `Err(err)` - The value could not be received.
    fn recv(&self) -> impl std::future::Future<Output = Result<Response<T>>>;
}