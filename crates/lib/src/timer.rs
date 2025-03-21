use std::fmt::{Display, Formatter};

use thiserror::Error;
use tokio::{
    sync::broadcast::{self, Receiver, Sender},
    time::{self, Duration, Interval},
};

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

/// A timer that counts down from a specified duration.
#[derive(Debug)]
pub struct Countdown {
    duration: Duration,
    interval: Interval,
    time_left_tx: Sender<Duration>,
}

/// Wraps a [`Duration`], formatting it as `HH:MM:SS`.
#[derive(Debug, PartialEq)]
pub struct DurationDisplay(Duration);

impl Default for Countdown {
    fn default() -> Self {
        let default_duration = Duration::from_secs(25 * 60);
        let default_interval = Duration::from_secs(1);

        Self::try_new(default_duration, default_interval).expect("failed to create default timer")
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
    pub fn try_new(duration: Duration, interval: Duration) -> Result<Self, InvalidCountdown> {
        // TODO: Validate inputs. Fail if interval > duration || interval == 0 || duration cannot be more than 1 day (86_400_000ms) u32
        if duration.is_zero() {
            return Err(InvalidCountdown::ZeroDuration);
        }

        if interval.is_zero() {
            return Err(InvalidCountdown::ZeroInterval);
        }

        if interval > duration {
            return Err(InvalidCountdown::IntervalGreaterThanDuration{duration: DurationDisplay(duration), interval: DurationDisplay(interval)});
        }

        if duration > Duration::from_secs(86_400) {
            return Err(InvalidCountdown::DurationGreaterThanOneDay(DurationDisplay(duration)));
        }

        if interval > Duration::from_secs(3600) {
            return Err(InvalidCountdown::IntervalGreaterThanOneHour(DurationDisplay(interval)));
        }

        let (time_left_tx, _) = broadcast::channel::<Duration>(1);
        Ok(Self { interval: time::interval(interval), time_left_tx, duration })
    }

    /// Starts the countdown.
    pub async fn start(&mut self) {
        let intervals = (self.duration.as_secs_f64() / self.interval.period().as_secs_f64()).ceil() as u32;

        for i in (0..=intervals).rev() {
            self.interval.tick().await;

            self.time_left_tx.send(self.interval.period() * i).unwrap(); // TODO: Handle failed send
        }
    }

    /// Watches the remaining time.
    pub fn watch(&self) -> Receiver<Duration> {
        self.time_left_tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::Duration;

    use super::*;

    #[test]
    fn should_fail_to_create_a_countdown_given_an_interval_greater_than_the_duration() {
        let result = Countdown::try_new(Duration::from_secs(1), Duration::from_secs(2));
        assert_eq!(result.expect_err("should have failed"), InvalidCountdown::IntervalGreaterThanDuration{duration: DurationDisplay(Duration::from_secs(1)), interval: DurationDisplay(Duration::from_secs(2))});
    }

    #[test]
    fn should_fail_to_create_a_countdown_given_a_duration_of_zero() {
        let result = Countdown::try_new(Duration::from_secs(0), Duration::from_millis(100));
        assert_eq!(result.expect_err("should have failed"), InvalidCountdown::ZeroDuration);
    }

    #[test]
    fn should_fail_to_create_a_countdown_given_an_interval_of_zero() {
        let result = Countdown::try_new(Duration::from_secs(1), Duration::from_millis(0));
        assert_eq!(result.expect_err("should have failed"), InvalidCountdown::ZeroInterval);
    }

    #[test]
    fn should_fail_to_create_a_countdown_given_a_duration_of_greater_than_one_day() {
        let result = Countdown::try_new(Duration::from_secs(86_401), Duration::from_millis(100));
        assert_eq!(result.expect_err("should have failed"), InvalidCountdown::DurationGreaterThanOneDay(DurationDisplay(Duration::from_secs(86_401))));
    }

    #[test]
    fn should_fail_to_create_a_countdown_given_an_interval_of_greater_than_one_hour() {
        let result = Countdown::try_new(Duration::from_secs(86_400), Duration::from_secs(3601));
        assert_eq!(result.expect_err("should have failed"), InvalidCountdown::IntervalGreaterThanOneHour(DurationDisplay(Duration::from_secs(3601))));
    }

    #[tokio::test]
    async fn should_countdown_to_zero() {
        let mut timer = Countdown::try_new(Duration::from_secs(1), Duration::from_millis(100)).expect("should have created countdown");
        let mut rx = timer.watch();

        let expectations = [
            1000u16, 900u16, 800u16, 700u16, 600u16, 500u16, 400u16, 300u16, 200u16, 100u16, 0u16,
        ]
        .map(|ms| Duration::from_millis(ms.into()));

        let countdown_handle = timer.start();
        let watch_handle = tokio::spawn(async move {
            for (i, expect) in expectations.iter().enumerate() {
                let actual = rx.recv().await.expect("Failed to receive duration").as_millis();
                assert_eq!(expect.as_millis(), actual, "Interval {} expected {}, but got {}", i, expect.as_millis(), actual);
            }
        });

        countdown_handle.await;
        watch_handle.abort();
    }
}
