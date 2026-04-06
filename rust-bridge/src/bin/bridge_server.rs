//! Long-lived HTTP server: JSON → CCSDS → UDP to CI_LAB.

use rust_bridge::server::run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    run().await
}
