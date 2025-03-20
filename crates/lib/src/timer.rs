use std::sync::RwLock;

use tokio::{
    sync::watch::{self, Receiver, Sender},
    time::{self, Duration, Instant, Interval},
};

/// A timer that counts down from a specified duration.
pub struct Countdown {
    duration: Duration,
    timer: RwLock<Interval>,
    tick_instant_tx: Sender<Instant>,
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
        let (tick_instant_tx, _) = watch::channel(Instant::now());
        let (time_left_tx, _) = watch::channel(duration);
        Self {
            timer: RwLock::new(time::interval(interval)),
            tick_instant_tx,
            time_left_tx,
            duration,
        }
    }

    /// Starts the countdown.
    pub async fn start(&mut self) {
        let mut instant_rx = self.tick_instant_tx.subscribe();
        let mut time_left = self.duration.clone();

        while !time_left.is_zero() {
            time_left -= self.tick(&mut instant_rx).await;

            self.time_left_tx.send_if_modified(try_set(&time_left));
        }
    }

    /// Watches the remaining time.
    pub fn watch(&self) -> Receiver<Duration> {
        self.time_left_tx.subscribe()
    }

    async fn tick(&mut self, instant_rx: &mut Receiver<Instant>) -> Duration {
        let last_instant = instant_rx.borrow_and_update();
        last_instant.duration_since(self.timer.write().unwrap().tick().await)
    }
}

fn try_set<T: Clone + PartialEq>(new: &T) -> impl FnMut(&mut T) -> bool {
    move |val| {
        if val == new {
            return false;
        }

        *val = new.clone();
        true
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

    #[tokio::test]
    async fn should_not_countdown_if_duration_is_zero() {
        let mut timer = Countdown::new(Duration::from_secs(0), Duration::from_millis(100));
        let mut rx = timer.watch();

        let countdown_handle = timer.start();
        let watch_handle = tokio::spawn(async move {
            rx.changed().await.unwrap();
            let actual = rx.borrow_and_update().as_millis();
            assert_eq!(0u128, actual);
        });

        countdown_handle.await;
        watch_handle.abort();
    }
}
