use thiserror::Error;

use crate::countdown::{
    timer::{InvalidCountdown, InvalidDuration}, 
    watcher::Zeroable,
};

mod timer;
mod watcher;

pub use timer::AsyncCountdown;
pub use watcher::ChannelWatcher;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error(transparent)]
    InvalidCountdown(#[from] InvalidCountdown),
    #[error(transparent)]
    InvalidDuration(#[from] InvalidDuration),
}

type Result<T> = std::result::Result<T, Error>;

/// A countdown that counts down from a specified duration.
pub trait Countdown {
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
    /// * `Ok(watcher)` - The countdown has started, and a [`ChannelWatcher`] is returned.
    /// * `Err(err)` - The countdown could not be started.
    fn start(&self, duration_millis: u64) -> impl std::future::Future<Output = Result<ChannelWatcher<u64>>>;
}

/// A watcher that receives updates from a countdown.
pub trait Watcher<T: Zeroable + Copy> {
    /// Returns the next value from the countdown.
    ///
    /// # Returns
    ///
    /// * `Some(value)` - The next value from the countdown.
    /// * `None` - The countdown has completed.
    fn next(&mut self) -> impl std::future::Future<Output = Option<T>>;
}