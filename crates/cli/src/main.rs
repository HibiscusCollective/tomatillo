use libtomatillo::{run, countdown::AsyncCountdown};

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let timer = AsyncCountdown::try_new(25000).expect("failed to create timer");

    run(timer, 1000).await;
}
