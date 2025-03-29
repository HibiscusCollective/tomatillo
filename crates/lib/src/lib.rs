use countdown::{Countdown, Receiver, Response};
use thiserror::Error;

pub mod view;
pub mod countdown;

#[derive(Debug, Error, PartialEq)]
pub enum TomatilloError {
    #[error(transparent)]
    CountdownError(#[from] crate::countdown::CountdownError),
    #[error(transparent)]
    ChannelError(#[from] crate::countdown::ChannelError),
}

pub async fn run(
    timer: impl Countdown<u64>,
    duration_millis: u64,
) {
    let countdown = timer.start(duration_millis).await.unwrap();

    while let Ok(Response::Value(millis_left)) = countdown.recv().await {
        println!("{:02}:{:02}", millis_left / 60, millis_left % 60);
    }
}

#[cfg(test)]
mod tests {
    // #[tokio::test(flavor = "multi_thread", worker_threads = 2)] TODO: Restore after we finish view
    // async fn should_display_countdown_as_it_changes() {

    //     let timer = AsyncCountdown::try_new(1000).expect("should have created timer");

    //     let expectations: &str = "00:03\r00:02\r00:01\r00:00\r";

    //     let mut buf = Vec::new();
    //     run(timer, &mut buf, 3000).await;

    //     assert_eq!(
    //         String::from_utf8(buf).unwrap(),
    //         expectations
    //     );
    // }
}