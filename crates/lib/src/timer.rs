use std::sync::Arc;

use tokio::{
    sync::{
        Mutex,
        watch::{self, Receiver, Sender},
    },
    time::{self, Duration, Instant, Interval},
};

/// A timer that counts down from a specified duration.
pub struct Timer {
    remaining: Arc<Mutex<Duration>>,
    interval: Mutex<Interval>,
    last_tick: Mutex<Instant>,
    tx: Sender<Duration>,
}

impl Default for Timer {
    fn default() -> Self {
        let default_duration = Duration::from_secs(25 * 60);
        let default_interval = Duration::from_secs(1);

        Self::new(default_duration, default_interval)
    }
}

impl Timer {
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
        let (tx, _) = watch::channel(duration);
        Self {
            remaining: Arc::new(Mutex::new(duration)),
            interval: Mutex::new(time::interval(interval)),
            last_tick: Mutex::new(Instant::now()),
            tx,
        }
    }

    /// Starts the countdown.
    pub async fn countdown(&mut self) {
        while !self.remaining.lock().await.is_zero() {
            let instant = self.interval.lock().await.tick().await;
            let delta = self.delta(instant).await;

            let rem = self.remaining.lock().await.checked_sub(delta).unwrap(); // TODO: Handle errors

            self.tx.send(rem).unwrap(); // TODO: Handle errors

            *self.remaining.lock().await = rem;
            *self.last_tick.lock().await = instant;
        }
    }

    /// Watches the remaining time.
    pub fn watch(&self) -> Receiver<Duration> {
        self.tx.subscribe()
    }

    async fn delta(&self, instant: Instant) -> Duration {
        instant.duration_since(*self.last_tick.lock().await)
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::Duration;

    use super::*;

    #[tokio::test]
    async fn should_countdown_to_zero() {
        let mut timer = Timer::new(Duration::from_secs(1), Duration::from_millis(100));
        let mut rx = timer.watch();

        let expectations = [
            1000u16, 900u16, 800u16, 700u16, 600u16, 500u16, 400u16, 300u16, 200u16, 100u16, 0u16,
        ]
        .map(|ms| Duration::from_millis(ms.into()));

        let countdown_handle = timer.countdown();
        let watch_handle = tokio::spawn(async move {
            for (i, expect) in expectations.iter().enumerate() {
                rx.changed().await.unwrap();
                let actual = rx.borrow_and_update().as_millis();
                assert_eq!(
                    expect.as_millis(),
                    actual,
                    "Interval {} expected {}, but got {}",
                    i,
                    expect.as_millis(),
                    actual
                );
            }
        });

        countdown_handle.await;
        watch_handle.abort();
    }
}
