use std::sync::Arc;

use tokio::{
    sync::{
        Mutex,
        watch::{self, Receiver, Sender},
    },
    time::{self, Duration, Interval},
};

/// A timer that counts down from a specified duration.
pub struct Timer {
    remaining: Arc<Mutex<Duration>>,
    interval: Mutex<Interval>,
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
            tx,
        }
    }

    /// Starts the countdown.
    pub async fn countdown(&mut self) {
        while !self.remaining.lock().await.is_zero() {
            self.interval.lock().await.tick().await;
            let rem = self
                .remaining
                .lock()
                .await
                .checked_sub(Duration::from_millis(100))
                .unwrap(); // TODO: Handle errors

            self.tx.send(rem).unwrap(); // TODO: Handle errors

            *self.remaining.lock().await = rem;
        }
    }

    /// Watches the remaining time.
    pub fn watch(&self) -> Receiver<Duration> {
        self.tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use tokio::time::Duration;

    use super::*;

    #[tokio::test]
    async fn should_countdown_to_zero() {
        let mut timer = Timer::new(Duration::from_secs(1), Duration::from_millis(100));
        let mut rx = timer.watch();

        let expectations = VecDeque::from([
            Duration::from_secs(1),
            Duration::from_millis(900),
            Duration::from_millis(800),
            Duration::from_millis(700),
            Duration::from_millis(600),
            Duration::from_millis(500),
            Duration::from_millis(400),
            Duration::from_millis(300),
            Duration::from_millis(200),
            Duration::from_millis(100),
            Duration::from_millis(0),
        ]);

        let countdown_handle = timer.countdown();
        let watch_handle = tokio::spawn(async move {
            for expect in expectations.iter() {
                assert_eq!(expect.as_millis(), rx.borrow_and_update().as_millis());
                rx.changed().await.unwrap();
            }
        });

        countdown_handle.await;
        watch_handle.abort();
    }
}
