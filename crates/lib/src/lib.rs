use std::{
    io::Write,
    sync::{Arc, RwLock},
};

// TODO: Use tokio::AsyncWrite
use tokio::{sync::watch::Receiver, time::Duration};

pub mod timer;

pub async fn run(writer: Arc<RwLock<impl Write + Send + Sync + 'static>>, mut timer: timer::Countdown) {
    let rx = timer.watch();
    let wr = writer.clone();
    let countdown_handle = timer.start();
    let watch_handle: tokio::task::JoinHandle<()> =
        tokio::spawn(async move { update_display(wr, rx).await });

    countdown_handle.await;

    writer.write().unwrap().write(b"00:00\r").unwrap();
    writer.write().unwrap().flush().unwrap();

    watch_handle.abort();
}

async fn update_display(
    writer: Arc<RwLock<impl Write + Send + 'static>>,
    mut rx: Receiver<Duration>,
) {
    loop {
        rx.changed().await.unwrap();
        let rem = rx.borrow_and_update().as_secs();
        write!(writer.write().unwrap(), "{:02}:{:02}\r", rem / 60, rem % 60).unwrap();
        writer.write().unwrap().flush().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn should_display_countdown_as_it_changes() {
        let timer = timer::Countdown::new(Duration::from_secs(5), Duration::from_secs(1));

        let expectations: &str = "00:05\r00:04\r00:03\r00:02\r00:01\r00:00\r";

        let buf = Arc::new(RwLock::new(Vec::new()));
        run(buf.clone(), timer).await;

        assert_eq!(
            String::from_utf8(buf.read().unwrap().clone()).unwrap(),
            expectations
        );
    }
}
