#![feature(fn_traits)]
#![feature(unboxed_closures)]

mod infrastructure;

use crate::infrastructure::log::init_logger;
use crate::infrastructure::start_server;
use log::{debug, error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();

    start_server().await?;

    debug!("This is a log message.");
    info!("This is a log message.");
    warn!("This is a log message.");
    error!("This is a log message.");

    Ok(())
}
