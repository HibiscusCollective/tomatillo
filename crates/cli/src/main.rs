use tokio::time::Duration;

use libtomatillo::{run, countdown};
use std::io::stdout;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let timer = countdown::Countdown::try_new(Duration::from_secs(25)).expect("failed to create timer");

    run(stdout(), timer).await;
}

