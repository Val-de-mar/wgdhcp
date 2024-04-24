use std::net::SocketAddr;
use tonic::transport::Server;

use crate::{common::config::CONFIG, service};

pub async fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::new(CONFIG.service.address, CONFIG.service.port);

    let service = service::ServiceImpl {};
    Server::builder()
        .add_service(service::DhcServiceServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}
