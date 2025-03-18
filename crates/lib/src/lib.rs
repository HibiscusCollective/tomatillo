use std::sync::Arc;

use tokio::{
    sync::{Mutex, mpsc::Sender},
    time::{self, Duration, Interval},
};

pub struct Timer {
    remaining: Arc<Mutex<Duration>>,
    interval: Mutex<Interval>,
    tx: Sender<Duration>,
}

impl Timer {
    pub fn new(duration: Duration, interval: Duration, tx: Sender<Duration>) -> Self {
        Self {
            remaining: Arc::new(Mutex::new(duration)),
            interval: Mutex::new(time::interval(interval)),
            tx,
        }
    }

    pub async fn countdown(&mut self) {
        while !self.remaining.lock().await.is_zero() {
            self.interval.lock().await.tick().await;
            let rem = self
                .remaining
                .lock()
                .await
                .checked_sub(Duration::from_millis(100))
                .unwrap();

            self.tx.send(rem).await.unwrap();

            *self.remaining.lock().await = rem;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use tokio::time::Duration;

    use super::*;

    #[tokio::test]
    async fn should_countdown_to_zero() {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Duration>(2);

        let mut timer = Timer::new(Duration::from_secs(1), Duration::from_millis(100), tx);
        let expect = Mutex::new(VecDeque::from([
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
        ]));

        tokio::spawn(async move {
            while let Some(rem) = rx.recv().await {
                assert_eq!(
                    rem.as_millis(),
                    expect.lock().await.pop_front().unwrap().as_millis()
                );
            }
        });

        timer.countdown().await;
    }
}
