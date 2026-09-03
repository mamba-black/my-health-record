mod infrastructure;

use crate::infrastructure::cli::Cli;
use crate::infrastructure::log::init_observability;
use crate::infrastructure::start_server;
use clap::Parser;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    init_observability();

    let cli = Cli::parse();
    let enable_admin_worker = cli.is_administration_worker_enabled();

    start_server(None, enable_admin_worker).await?;

    Ok(())
}
