use std::sync::Arc;

use thiserror::Error;
use tokio::{
    sync::{watch::{self, Receiver}, Mutex},
    time::{self, Duration, Interval},
};

#[derive(Debug, Error, PartialEq)]
pub enum Error {
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
    DurationSmallerThanInterval{duration: Duration, period: Duration},
}

/// A countdown that counts down from a specified duration.
#[derive(Debug)]
pub struct Countdown {
    interval: Arc<Mutex<Interval>>,
}

#[derive(Debug)]
pub struct Watcher {
    rx: Receiver<Duration>,
}

impl Default for Countdown {
    fn default() -> Self {
        const DEFAULT_PERIOD: Duration = Duration::from_secs(1);
        Self::try_new(DEFAULT_PERIOD).expect("failed to create default timer")
    }
}

impl Countdown {
    /// Creates a new [`Countdown`].
    ///
    /// # Arguments
    ///
    /// * `period` - The interval at which the timer should be updated.
    ///
    /// # Returns
    ///
    /// A new [`Countdown`].
    pub fn try_new(period: Duration) -> Result<Self, Error> {
        validate_interval(period)?;

        Ok(Self { interval: Arc::new(Mutex::new(time::interval(period))) })
    }

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
    /// * `Ok(watcher)` - The countdown has started, and a [`Watcher`] is returned.
    /// * `Err(err)` - The countdown could not be started.
    pub async fn start(&mut self, duration: Duration) -> Result<Watcher, Error> {
        self.validate_duration(duration).await?;
        
        let (tx, rx) = watch::channel(duration);
        
        tokio::spawn(countdown(self.interval.clone(), tx, duration));

        Ok(Watcher { rx })
    }

    async fn validate_duration(&self, duration: Duration) -> Result<(), InvalidDuration> {
        if duration.is_zero() {
            return Err(InvalidDuration::ZeroDuration);
        }
    
        if duration > Duration::from_secs(86_400) {
            return Err(InvalidDuration::DurationGreaterThanOneDay(duration));
        }

        let period = self.interval.lock().await.period();
        if period > duration {
            return Err(InvalidDuration::DurationSmallerThanInterval{duration, period});
        }
    
        Ok(())
    }
}

async fn countdown(interval: Arc<Mutex<Interval>>, tx: watch::Sender<Duration>, duration: Duration) {
    let period = interval.lock().await.period();
    let intervals = calc_intervals(duration, period);

    for i in (0..=intervals).rev() {
        interval.lock().await.tick().await;
    
        tx.send(period * i).unwrap();
    }
}

impl Watcher {
    pub async fn next(&mut self) -> Option<Duration> {
        self.rx.changed().await.expect("sender was unexpectedly dropped");

        let val = self.rx.borrow_and_update();
        if val.is_zero() {
            return None;
        }

        Some(val.clone())
    }
}

fn validate_interval(interval: Duration) -> Result<(), InvalidCountdown> {
    if interval.is_zero() {
        return Err(InvalidCountdown::ZeroInterval);
    }

    if interval > Duration::from_secs(3600) {
        return Err(InvalidCountdown::IntervalGreaterThanOneHour(interval));
    }

    Ok(())
}

fn calc_intervals(duration: Duration, period: Duration) -> u32 {
    (duration.as_secs_f64() / period.as_secs_f64()).ceil() as u32
}

#[cfg(test)]
mod tests {
    use tokio::time::Duration;

    use super::*;

    #[tokio::test]
    async fn should_fail_to_create_a_countdown_given_an_interval_of_zero() {
        let error = Countdown::try_new(Duration::from_millis(0)).expect_err("should have failed");
        assert_eq!(error, Error::InvalidCountdown(InvalidCountdown::ZeroInterval));
    }

    #[tokio::test]
    async fn should_fail_to_create_a_countdown_given_an_interval_of_greater_than_one_hour() {
        let result = Countdown::try_new(Duration::from_secs(3601)).expect_err("should have failed");
        assert_eq!(result, Error::InvalidCountdown(InvalidCountdown::IntervalGreaterThanOneHour(Duration::from_secs(3601))));
    }

    #[tokio::test]
    async fn should_fail_to_start_a_countdown_given_an_duration_smaller_than_the_interval() {
        let error = Countdown::try_new(Duration::from_secs(2)).expect("unexpected error creating a countdown")
            .start(Duration::from_secs(1)).await.expect_err("should have failed to start");
        assert_eq!(error, Error::InvalidDuration(InvalidDuration::DurationSmallerThanInterval{duration: Duration::from_secs(1), period: Duration::from_secs(2)}));
    }

    #[tokio::test]
    async fn should_fail_to_start_a_countdown_given_a_duration_of_zero() {
        let error = Countdown::try_new(Duration::from_millis(100)).expect("unexpected error creating a countdown")
            .start(Duration::ZERO).await.expect_err("should have failed to start");
        assert_eq!(error, Error::InvalidDuration(InvalidDuration::ZeroDuration));
    }

    #[tokio::test]
    async fn should_fail_to_start_a_countdown_given_a_duration_of_greater_than_one_day() {
        let error = Countdown::try_new(Duration::from_millis(100)).expect("unexpected error creating a countdown")
            .start(Duration::from_secs(86_401)).await.expect_err("should have failed to start");
        assert_eq!(error, Error::InvalidDuration(InvalidDuration::DurationGreaterThanOneDay(Duration::from_secs(86_401))));
    }

    #[tokio::test]
    async fn should_countdown_to_zero() {
        let mut timer = Countdown::try_new(Duration::from_secs(1)).expect("should have created countdown");
        let mut expectations = [1000u16, 900u16, 800u16, 700u16, 600u16, 500u16, 400u16, 300u16, 200u16, 100u16, 0u16].map(|ms| Duration::from_millis(ms.into())).to_vec();
        expectations.reverse();

        let mut watcher = timer.start(Duration::from_secs(1)).await.expect("unexpected countdown failure");
        while let Some(time_left) = watcher.next().await {
            if let Some(expect) = expectations.pop() {
                assert_eq!(expect, time_left, "expected {:?}, but got {:?}", expect, time_left);
            } else {
                assert!(false, "unexpected time left received: {:?}", time_left)
            }
        }
    }
}