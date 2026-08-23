mod infrastructure;

use clap::Parser;
use crate::infrastructure::cli::Cli;
use crate::infrastructure::log::init_logger;
use crate::infrastructure::start_server;
use dotenvy::dotenv;
use log::{debug, error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();
    dotenv().ok();

    let cli = Cli::parse();
    let enable_admin_worker = cli.is_administration_worker_enabled();

    start_server(None, enable_admin_worker).await?;

    debug!("This is a log message.");
    info!("This is a log message.");
    warn!("This is a log message.");
    error!("This is a log message.");

    Ok(())
}
