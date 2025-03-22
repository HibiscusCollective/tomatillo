use std::fmt::{Display, Formatter};
use thiserror::Error;
use tokio::{
    sync::broadcast::{self, Receiver, Sender},
    time::{self, Duration, Interval},
};
use tokio::sync::broadcast::error::SendError;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    InvalidCountdown(#[from] InvalidCountdown),
    #[error(transparent)]
    CountdownError(#[from] SendError<Duration>)
}

impl PartialEq for Error {
    fn eq(&self, other: &Error) -> bool {
        match (self, other) {
            (Error::InvalidCountdown(_), Error::InvalidCountdown(_)) => true,
            (Error::CountdownError(err), Error::CountdownError(other)) => {
                err.0 == other.0
            },
            _ => false
        }
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum InvalidCountdown {
    #[error("Interval {interval} cannot be greater than duration {duration}")]
    IntervalGreaterThanDuration{duration: DurationDisplay, interval: DurationDisplay},
    #[error("Duration cannot be zero")]
    ZeroDuration,
    #[error("Interval cannot be zero")]
    ZeroInterval,
    #[error("Duration {0} cannot be greater than one day")]
    DurationGreaterThanOneDay(DurationDisplay),
    #[error("Interval {0} cannot be greater than one hour")]
    IntervalGreaterThanOneHour(DurationDisplay),
}

/// A countdown that counts down from a specified duration.
#[derive(Debug)]
pub struct Countdown {
    duration: Duration,
    interval: Interval,
    time_left_tx: Sender<Duration>,
    send_retry_limit: u8,
}

/// Wraps a [`Duration`], formatting it as `HH:MM:SS`.
#[derive(Debug, PartialEq)]
pub struct DurationDisplay(Duration);

impl Default for Countdown {
    fn default() -> Self {
        let default_duration = Duration::from_secs(25 * 60);
        let default_interval = Duration::from_secs(1);
        const DEFAULT_SEND_RETRY_LIMIT: u8 = 5;

        Self::try_new(default_duration, default_interval, DEFAULT_SEND_RETRY_LIMIT).expect("failed to create default timer")
    }
}

impl Display for DurationDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.0.as_secs() / 3600, (self.0.as_secs() / 60) % 60, self.0.as_secs() % 60)
    }
}

impl Countdown {
    /// Creates a new [`Timer`].
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration of the timer.
    /// * `interval` - The interval at which the timer should be updated.
    ///
    /// # Returns
    ///
    /// A new [`Timer`].
    pub fn try_new(duration: Duration, interval: Duration, send_retry_limit: u8) -> Result<Self, Error> {
        validate_duration(duration)?;
        validate_interval(interval)?;

        if interval > duration {
            return Err(Error::InvalidCountdown(InvalidCountdown::IntervalGreaterThanDuration{duration: DurationDisplay(duration), interval: DurationDisplay(interval)}));
        }

        let (time_left_tx, _) = broadcast::channel::<Duration>(1);
        Ok(Self { interval: time::interval(interval), time_left_tx, duration, send_retry_limit })
    }

    /// Starts the countdown.
    pub async fn start(&mut self) -> Result<(), Error> {
        let intervals = (self.duration.as_secs_f64() / self.interval.period().as_secs_f64()).ceil() as u32;

        let mut retries_left = self.send_retry_limit;
        for i in (0..=intervals).rev() {
            self.interval.tick().await;
            let time_left = self.interval.period() * i;
            print!("{:?}", time_left);

            match self.time_left_tx.send(time_left) {
                Err(e) => {
                    if retries_left == 0 {
                        return Err(Error::CountdownError(e));
                    } else {
                        retries_left -= 1;
                    }
                }
                _ => { retries_left = self.send_retry_limit; }
            }

            println!("-> {:?}", time_left);
        }
        Ok(())
    }

    /// Watches the remaining time.
    pub fn watch(&self) -> Receiver<Duration> {
        self.time_left_tx.subscribe()
    }
}

fn validate_duration(duration: Duration) -> Result<(), InvalidCountdown> {
    if duration.is_zero() {
        return Err(InvalidCountdown::ZeroDuration);
    }

    if duration > Duration::from_secs(86_400) {
        return Err(InvalidCountdown::DurationGreaterThanOneDay(DurationDisplay(duration)));
    }

    Ok(())
}

fn validate_interval(interval: Duration) -> Result<(), InvalidCountdown> {
    if interval.is_zero() {
        return Err(InvalidCountdown::ZeroInterval);
    }

    if interval > Duration::from_secs(3600) {
        return Err(InvalidCountdown::IntervalGreaterThanOneHour(DurationDisplay(interval)));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::{select, sync::Mutex, time::Duration};

    use super::*;

    #[test]
    fn should_fail_to_create_a_countdown_given_an_interval_greater_than_the_duration() {
        let result = Countdown::try_new(Duration::from_secs(1), Duration::from_secs(2), 0);
        assert_eq!(result.expect_err("should have failed"), Error::InvalidCountdown(InvalidCountdown::IntervalGreaterThanDuration{duration: DurationDisplay(Duration::from_secs(1)), interval: DurationDisplay(Duration::from_secs(2))}));
    }

    #[test]
    fn should_fail_to_create_a_countdown_given_a_duration_of_zero() {
        let result = Countdown::try_new(Duration::from_secs(0), Duration::from_millis(100), 0);
        assert_eq!(result.expect_err("should have failed"), Error::InvalidCountdown(InvalidCountdown::ZeroDuration));
    }

    #[test]
    fn should_fail_to_create_a_countdown_given_an_interval_of_zero() {
        let result = Countdown::try_new(Duration::from_secs(1), Duration::from_millis(0), 0);
        assert_eq!(result.expect_err("should have failed"), Error::InvalidCountdown(InvalidCountdown::ZeroInterval));
    }

    #[test]
    fn should_fail_to_create_a_countdown_given_a_duration_of_greater_than_one_day() {
        let result = Countdown::try_new(Duration::from_secs(86_401), Duration::from_millis(100), 0);
        assert_eq!(result.expect_err("should have failed"), Error::InvalidCountdown(InvalidCountdown::DurationGreaterThanOneDay(DurationDisplay(Duration::from_secs(86_401)))));
    }

    #[test]
    fn should_fail_to_create_a_countdown_given_an_interval_of_greater_than_one_hour() {
        let result = Countdown::try_new(Duration::from_secs(86_400), Duration::from_secs(3601), 0);
        assert_eq!(result.expect_err("should have failed"), Error::InvalidCountdown(InvalidCountdown::IntervalGreaterThanOneHour(DurationDisplay(Duration::from_secs(3601)))));
    }

    #[tokio::test]
    async fn should_skip_a_tick_when_the_new_duration_could_not_be_sent() {
        time::pause();
        let timer = Arc::new(Mutex::new(Countdown::try_new(Duration::from_millis(500), Duration::from_millis(100), 5).expect("should have created countdown")));

        let counter = timer.clone();
        let countdown_handle = tokio::spawn(async move {
            counter.lock().await.start().await.expect("unexpected countdown failure");
        });

        time::advance(Duration::from_millis(300)).await;
        time::resume();

        let watch_handle = tokio::spawn(async move {
            let mut rx = timer.lock().await.watch();
            for (i, expect) in [200u128, 100u128, 0u128].iter().cloned().enumerate() {
                let actual = rx.recv().await.expect("Failed to receive duration").as_millis();
                assert_eq!(expect, actual, "Interval {} expected {}, but got {}", i, expect, actual);

                if actual == 0 {
                    return;
                }
            }
        });
        
        select! {
            _ = countdown_handle => {}
            _ = watch_handle => {}
        }
    }

    #[tokio::test]
    async fn should_countdown_to_zero() {
        let mut timer = Countdown::try_new(Duration::from_secs(1), Duration::from_millis(100), 0).expect("should have created countdown");
        let mut rx = timer.watch();

        let expectations = [1000u16, 900u16, 800u16, 700u16, 600u16, 500u16, 400u16, 300u16, 200u16, 100u16, 0u16].map(|ms| Duration::from_millis(ms.into()));

        let countdown_handle = tokio::spawn(async move {
            timer.start().await.expect("unexpected countdown failure");
        });
        let watch_handle = tokio::spawn(async move {
            for (i, expect) in expectations.iter().enumerate() {
                let actual = rx.recv().await.expect("Failed to receive duration").as_millis();
                assert_eq!(expect.as_millis(), actual, "Interval {} expected {}, but got {}", i, expect.as_millis(), actual);
            }
        });

        countdown_handle.await.expect("unexpected error in countdown");
        watch_handle.abort();
    }

    #[tokio::test]
    async fn should_stop_retrying_sending_after_several_failed_attempts() {
        time::pause();
        let timer = Arc::new(Mutex::new(Countdown::try_new(Duration::from_secs(1), Duration::from_millis(100), 5).expect("should have created countdown")));
        
        let err = timer.lock().await.start().await.expect_err("should have failed");
        assert_eq!(err, Error::CountdownError(SendError(Duration::from_millis(500))))
    }
}