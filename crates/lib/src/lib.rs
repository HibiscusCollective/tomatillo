use std::io::Write;

use countdown::{Countdown, Watcher};

pub mod display;
pub mod countdown;

pub async fn run(
    timer: impl Countdown,
    mut writer: impl Write,
    duration_millis: u64,
) {
    let mut countdown = timer.start(duration_millis).await.expect("unexpected countdown failure");
    
    while let Some(time_left) = countdown.next().await {
        write!(writer, "{:?}\r", time_left).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use countdown::AsyncCountdown;

    use super::*;

//     #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
//     async fn should_display_countdown_as_it_changes() {

//         let timer = AsyncCountdown::try_new(1000).expect("should have created timer");

//         let expectations: &str = "00:03\r00:02\r00:01\r00:00\r";

//         let mut buf = Vec::new();
//         run(timer, &mut buf, 3000).await;

//         assert_eq!(
//             String::from_utf8(buf).unwrap(),
//             expectations
//         );
//     }
// }
}