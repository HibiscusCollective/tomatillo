use std::{ops::Mul, sync::Arc};

use thiserror::Error;
use tokio::{
    sync::{watch, Mutex},
    time::{self, Duration, Interval},
};

use super::{watcher::ChannelWatcher, Countdown, Result};

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
    interval: Arc<Mutex<Interval>>,
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
            return Err(InvalidDuration::ZeroDuration.into());
        }
    
        if duration > 86_400 {
            return Err(InvalidDuration::DurationGreaterThanOneDay(Duration::from_millis(duration)).into());
        }

        let period = self.interval.lock().await.period();
        if period > Duration::from_millis(duration) {
            return Err(InvalidDuration::DurationSmallerThanPeriod{duration: Duration::from_millis(duration), period: period.clone()}.into());
        }
    
        Ok(())
    }
}

impl Countdown for AsyncCountdown {
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
    /// * `Ok(watcher)` - The countdown has started, and a [`ChannelWatcher`] is returned.
    /// * `Err(err)` - The countdown could not be started.
    async fn start(&self, duration_millis: u64) -> Result<ChannelWatcher<u64>> {
        self.validate_duration(duration_millis).await?;
        
        let (tx, rx) = watch::channel(duration_millis);
        
        tokio::spawn(countdown(self.interval.clone(), tx, duration_millis));

        Ok(ChannelWatcher::from(rx))
    }
}
async fn countdown(interval: Arc<Mutex<Interval>>, tx: watch::Sender<u64>, duration: u64) {
    let period = &interval.lock().await.period();
    let intervals = calc_intervals(Duration::from_millis(duration), period);

    for i in (0..=intervals).rev() {
        interval.lock().await.tick().await;
    
        tx.send(period.mul(i).as_millis() as u64).unwrap();
    }
}

fn validate_period(period: u64) -> Result<()> {
    if period == 0u64 {
        return Err(InvalidCountdown::ZeroInterval.into());
    }

    if period > 3600 {
        return Err(InvalidCountdown::IntervalGreaterThanOneHour(Duration::from_millis(period)).into());
    }

    Ok(())
}

fn calc_intervals(duration: Duration, period: &Duration) -> u32 {
    (duration.as_secs_f64() / period.as_secs_f64()).ceil() as u32
}

#[cfg(test)]
mod tests {
    use tokio::time::Duration;

    use crate::countdown::Watcher;

    use super::*;

    #[tokio::test]
    async fn should_fail_to_create_a_countdown_given_an_interval_of_zero() {
        let error = AsyncCountdown::try_new(0).expect_err("should have failed");
        assert_eq!(error, InvalidCountdown::ZeroInterval.into());
    }

    #[tokio::test]
    async fn should_fail_to_create_a_countdown_given_an_interval_of_greater_than_one_hour() {
        let result = AsyncCountdown::try_new(3601).expect_err("should have failed");
        assert_eq!(result, InvalidCountdown::IntervalGreaterThanOneHour(Duration::from_millis(3601)).into());
    }

    #[tokio::test]
    async fn should_fail_to_start_a_countdown_given_an_duration_smaller_than_the_interval() {
        let error = AsyncCountdown::try_new(2000).expect("unexpected error creating a countdown")
            .start(1000).await.expect_err("should have failed to start");
        assert_eq!(error, InvalidDuration::DurationSmallerThanPeriod{duration: Duration::from_millis(1000), period: Duration::from_millis(2000)}.into());
    }

    #[tokio::test]
    async fn should_fail_to_start_a_countdown_given_a_duration_of_zero() {
        let error = AsyncCountdown::try_new(100).expect("unexpected error creating a countdown")
            .start(0).await.expect_err("should have failed to start");
        assert_eq!(error, InvalidDuration::ZeroDuration.into());
    }

    #[tokio::test]
    async fn should_fail_to_start_a_countdown_given_a_duration_of_greater_than_one_day() {
        let error = AsyncCountdown::try_new(100).expect("unexpected error creating a countdown")
            .start(86_401).await.expect_err("should have failed to start");
        assert_eq!(error, InvalidDuration::DurationGreaterThanOneDay(Duration::from_millis(86_401)).into());
    }

    #[tokio::test]
    async fn should_countdown_to_zero() {
        let timer = AsyncCountdown::try_new(1000).expect("should have created countdown");
        let mut expectations = [1000u64, 900u64, 800u64, 700u64, 600u64, 500u64, 400u64, 300u64, 200u64, 100u64, 0u64].iter().rev().cloned().collect::<Vec<_>>();

        let mut watcher = timer.start(1000).await.expect("unexpected countdown failure");
        while let Some(millis_left) = watcher.next().await {
            if let Some(expect) = expectations.pop() {
                assert_eq!(expect, millis_left, "expected {:?}, but got {:?}", expect, millis_left);
            } else {
                assert!(false, "unexpected time left received: {:?}", millis_left)
            }
        }
    }
}