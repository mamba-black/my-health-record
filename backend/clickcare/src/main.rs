mod infrastructure;

use log::{debug, error, info, warn};
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use crate::infrastructure::api::*;
use crate::infrastructure::api::patient_service_server::*;
use crate::infrastructure::log::init_logger;
use crate::infrastructure::start_server;

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
