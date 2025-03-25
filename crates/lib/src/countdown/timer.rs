use std::sync::Arc;

use thiserror::Error;
use tokio::{
    sync::Mutex,
    time::{self, Duration, Interval},
};

use super::{channel::{Channel, ChannelReceiver}, Countdown, Result, Sender};

#[derive(Debug, Error, PartialEq)]
pub enum TimerError {
    #[error(transparent)]
    InvalidCountdown(#[from] InvalidCountdown),
    #[error(transparent)]
    InvalidDuration(#[from] InvalidDuration),
}

#[derive(Debug, Error, PartialEq)]
pub enum InvalidCountdown {
    #[error("Interval cannot be zero")]
    ZeroInterval,
    #[error("Interval {0:?} cannot be greater than one hour")]
    IntervalGreaterThanOneHour(Duration),
}

#[derive(Debug, Error, PartialEq)]
pub enum InvalidDuration {
    #[error("Duration cannot be zero")]
    ZeroDuration,
    #[error("Duration {0:?} cannot be greater than one day")]
    DurationGreaterThanOneDay(Duration),
    #[error("Duration {duration:?} cannot be smaller than period {period:?}")]
    DurationSmallerThanPeriod{duration: Duration, period: Duration},
}

/// A countdown that counts down from a specified duration.
#[derive(Debug)]
pub struct AsyncCountdown {
    interval: Arc<Mutex<Interval>>
}

impl Default for AsyncCountdown {
    fn default() -> Self {
        const DEFAULT_PERIOD: u64 = 1000;
        Self::try_new(DEFAULT_PERIOD).expect("failed to create default timer")
    }
}

impl AsyncCountdown {
    /// Creates a new [`Countdown`] timer.
    ///
    /// # Arguments
    ///
    /// * `period` - The interval at which the timer should be updated.
    ///
    /// # Returns
    ///
    /// A [`Result`] that is:
    ///
    /// * `Ok(timer)` - The countdown timer has been created.
    /// * `Err(err)` - The countdown timer could not be created.
    pub fn try_new(period_millis: u64) -> Result<Self> {
        validate_period(period_millis)?;

        Ok(Self { interval: Arc::new(Mutex::new(time::interval(Duration::from_millis(period_millis)))) })
    }

    async fn validate_duration(&self, duration: u64) -> Result<()> {
        if duration == 0 {
            return Err(TimerError::InvalidDuration(InvalidDuration::ZeroDuration).into());
        }
    
        if duration > 86_400 {
            return Err(TimerError::InvalidDuration(InvalidDuration::DurationGreaterThanOneDay(Duration::from_millis(duration))).into());
        }

        let period = self.interval.lock().await.period();
        if period > Duration::from_millis(duration) {
            return Err(TimerError::InvalidDuration(InvalidDuration::DurationSmallerThanPeriod{duration: Duration::from_millis(duration), period: period.clone()}).into());
        }
    
        Ok(())
    }
}

impl Countdown<u64> for AsyncCountdown {
    /// Starts the countdown.
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration of the countdown.
    ///
    /// # Returns
    ///
    /// A [`Result`] that is:
    ///
    /// * `Ok(watcher)` - The countdown has started, and a [`ChannelReceiver`] is returned.
    /// * `Err(err)` - The countdown could not be started.
    async fn start(&self, duration_millis: u64) -> Result<ChannelReceiver<u64>> {
        self.validate_duration(duration_millis).await?;
        
        let (tx, rx) = Channel::new(duration_millis);   
        tokio::spawn(countdown(self.interval.clone(), tx, duration_millis));

        Ok(rx)
    }
}

async fn countdown(interval: Arc<Mutex<Interval>>, tx: impl Sender<u64>, duration: u64) {
    let period = &interval.lock().await.period();
    let intervals = calc_intervals(Duration::from_millis(duration), period);
    let period_ms = period.as_millis() as u64;

    for i in 0..=intervals {
        interval.lock().await.tick().await;

        tx.send(duration - (period_ms * i as u64)).await.expect("unexpected error sending value");
    }

    tx.close().await.expect("unexpected error closing channel");
}

fn validate_period(period: u64) -> Result<()> {
    if period == 0u64 {
        return Err(TimerError::InvalidCountdown(InvalidCountdown::ZeroInterval).into());
    }

    if period > 3600 * 1000 {
        return Err(TimerError::InvalidCountdown(InvalidCountdown::IntervalGreaterThanOneHour(Duration::from_millis(period))).into());
    }

    Ok(())
}

fn calc_intervals(duration: Duration, period: &Duration) -> u32 {
    (duration.as_secs_f64() / period.as_secs_f64()).ceil() as u32
}

#[cfg(test)]
mod tests {
    use tokio::time::Duration;

    use crate::countdown::{Receiver, Response};

    use super::*;

    const HOUR_MS: u64 = 60u64 * 60 * 1000;
    const DAY_MS: u64 = 24u64 * 60 * 60 * 1000;

    #[tokio::test]
    async fn should_fail_to_create_a_countdown_given_a_period_of_zero() {
        let error = AsyncCountdown::try_new(0).expect_err("should have failed");
        assert_eq!(error, TimerError::InvalidCountdown(InvalidCountdown::ZeroInterval).into());
    }

    #[tokio::test]
    async fn should_fail_to_create_a_countdown_given_a_period_of_greater_than_one_hour() {
        let result = AsyncCountdown::try_new(HOUR_MS + 1).expect_err("should have failed");
        assert_eq!(result, TimerError::InvalidCountdown(InvalidCountdown::IntervalGreaterThanOneHour(Duration::from_millis(HOUR_MS + 1))).into());
    }

    #[tokio::test]
    async fn should_fail_to_start_a_countdown_given_an_duration_smaller_than_the_interval() {
        let error = AsyncCountdown::try_new(2000).expect("unexpected error creating a countdown")
            .start(1000).await.expect_err("should have failed to start");
        assert_eq!(error, TimerError::InvalidDuration(InvalidDuration::DurationSmallerThanPeriod{duration: Duration::from_millis(1000), period: Duration::from_millis(2000)}).into());
    }

    #[tokio::test]
    async fn should_fail_to_start_a_countdown_given_a_duration_of_zero() {
        let error = AsyncCountdown::try_new(100).expect("unexpected error creating a countdown")
            .start(0).await.expect_err("should have failed to start");
        assert_eq!(error, TimerError::InvalidDuration(InvalidDuration::ZeroDuration).into());
    }

    #[tokio::test]
    async fn should_fail_to_start_a_countdown_given_a_duration_of_greater_than_one_day() {
        let error = AsyncCountdown::try_new(100).expect("unexpected error creating a countdown")
            .start(DAY_MS + 1).await.expect_err("should have failed to start");
        assert_eq!(error, TimerError::InvalidDuration(InvalidDuration::DurationGreaterThanOneDay(Duration::from_millis(DAY_MS + 1))).into());
    }

    #[tokio::test]
    async fn should_countdown_to_zero() {
        time::pause();
        let timer = AsyncCountdown::try_new(100).expect("should have created countdown");
        let mut expectations = [1000u64, 900u64, 800u64, 700u64, 600u64, 500u64, 400u64, 300u64, 200u64, 100u64, 0u64].iter().rev().cloned().collect::<Vec<_>>();
        let num_expect = expectations.len();

        let rx = timer.start(1000).await.expect("unexpected countdown failure");

        while let Some(expect) = expectations.pop() {
            if let Ok(Response::Value(millis_left)) = rx.recv().await {
                assert_eq!(expect, millis_left, "[{:?}] expected {:?}, but got {:?}", num_expect - expectations.len(), expect, millis_left);
            }
            time::advance(Duration::from_millis(100u64)).await;
        }

        assert_eq!(expectations.len(), 0, "unmet expectations: {:?}", expectations.iter().rev().collect::<Vec<_>>());
    }
}