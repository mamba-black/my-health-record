mod infrastructure;

use clap::Parser;
use crate::infrastructure::cli::Cli;
use crate::infrastructure::log::init_observability;
use crate::infrastructure::start_server;
use dotenvy::dotenv;
use tracing::{debug, error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    init_observability();

    let cli = Cli::parse();
    let enable_admin_worker = cli.is_administration_worker_enabled();

    start_server(None, enable_admin_worker).await?;

    Ok(())
}
