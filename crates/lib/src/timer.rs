use tokio::{
    sync::broadcast::{self, Receiver, Sender},
    time::{self, Duration, Interval},
};

/// A timer that counts down from a specified duration.
pub struct Countdown {
    duration: Duration,
    interval: Interval,
    time_left_tx: Sender<Duration>,
}

impl Default for Countdown {
    fn default() -> Self {
        let default_duration = Duration::from_secs(25 * 60);
        let default_interval = Duration::from_secs(1);

        Self::new(default_duration, default_interval)
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
    pub fn new(duration: Duration, interval: Duration) -> Self {
        // TODO: Validate inputs. Fail if interval > duration || interval == 0 || duration cannot be more than 1 day (86_400_000ms) u32

        let (time_left_tx, _) = broadcast::channel::<Duration>(1);
        Self { interval: time::interval(interval), time_left_tx, duration }
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

    #[tokio::test]
    async fn should_countdown_to_zero() {
        let mut timer = Countdown::new(Duration::from_secs(1), Duration::from_millis(100));
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

    #[tokio::test]
    async fn should_not_countdown_if_duration_is_zero() {
        let mut timer = Countdown::new(Duration::from_secs(0), Duration::from_millis(100));
        let mut rx = timer.watch();

        let countdown_handle = timer.start();
        let watch_handle = tokio::spawn(async move {
            let actual = rx.recv().await.expect("Failed to receive duration").as_millis();
            assert_eq!(0u128, actual);
        });

        countdown_handle.await;
        watch_handle.abort();
    }
}
