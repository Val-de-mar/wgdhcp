use std::net::SocketAddr;
use std::process::Stdio;
use ipnet::IpNet;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tonic::transport::Server;
use crate::common::storage::get_storage;
use crate::common::wg::{IntoBase64, PrivateKey};

use crate::{common::config::CONFIG, service};

pub async fn setup_wireguard_interface(private_key: &PrivateKey) -> Result<(), Box<dyn std::error::Error>> {
    // Создание интерфейса wg0
    Command::new("ip")
        .args(["link", "add", &CONFIG.interface, "type", "wireguard"])
        .status()
        .await?;

    // Назначение IP адреса интерфейсу wg0
    Command::new("ip")
        .args(["address", "add", &CONFIG.internal_address.to_string(), "dev", &CONFIG.interface])
        .status()
        .await?;

    // Настройка приватного ключа через /dev/stdin
    let mut child = Command::new("wg")
        .args(["set", &CONFIG.interface, "private-key", "/dev/stdin"])
        .stdin(Stdio::piped())
        .spawn()?;
    
    let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;
    stdin.write_all(private_key.into_base_64().as_bytes()).await?;
    stdin.flush().await?;
    let status = child.wait().await?;

    // Поднятие интерфейса wg0
    Command::new("ip")
        .args(["link", "set", "up", "dev", &CONFIG.interface])
        .status()
        .await?;

    if !status.success() {
        return Err("Failed to set private key".into());
    }

    Ok(())
}

pub async fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::new(CONFIG.service.address, CONFIG.service.port);

    {
        let storage = get_storage().await;
        setup_wireguard_interface(&storage.interface.private_key);
    }
    let service = service::ServiceImpl {};
    Server::builder()
        .add_service(service::DhcServiceServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}
