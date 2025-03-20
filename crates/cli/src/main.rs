use std::{sync::{Arc, RwLock}, time::Duration};

use libtomatillo::{run, timer};
use std::io::stdout;

#[tokio::main]
async fn main() {
    let timer = timer::Countdown::new(Duration::from_secs(25), Duration::from_secs(1));
    let writer = Arc::new(RwLock::new(stdout()));
    
    run(writer, timer).await;
}
