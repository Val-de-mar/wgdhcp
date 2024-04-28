use std::process::{ExitStatus, Stdio};

use clap::Args;
use ipnet::IpNet;
use tokio::{io::AsyncWriteExt, process::Command};

use crate::common::wg::FromBase64;
use crate::common::wg::{self, IntoBase64, KeyPair, PrivateKey};
use tonic::transport::channel::Endpoint as TEndpoint;

use crate::common::proto::{dhc_service_client::DhcServiceClient, ReserveIpRequest};

#[derive(Debug, Args)]
pub struct Arguments {
    #[clap(help = "wg dhc server endpoint, including http or https protocole and port")]
    pub host: String,
    #[clap(help = "any string that identifies you for admin's conviniece")]
    pub account: String,
    #[clap(default_value_t={"wg0".to_string()}, help="wg interface name to be created")]
    pub interface: String,
    #[clap(default_value_t={5}, help="persistent_keepalive parameter for wireguard")]
    pub persistent_keepalive: usize,
}

fn check(status: ExitStatus, error: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !status.success() {
        return Err(error.into());
    }
    Ok(())
}

pub async fn setup_wireguard_interface(
    private_key: &PrivateKey,
    internal_address: &str,
    args: &Arguments,
) -> Result<(), Box<dyn std::error::Error>> {
    // Создание интерфейса wg0
    check(
        Command::new("ip")
            .args(["link", "add", &args.interface, "type", "wireguard"])
            .status()
            .await?,
        "Failed to add interface",
    )?;

    // Назначение IP адреса интерфейсу wg0
    check(
        Command::new("ip")
            .args(["address", "add", internal_address, "dev", &args.interface])
            .status()
            .await?,
        "Failed to set ip to interface",
    )?;

    // Настройка приватного ключа через /dev/stdin
    let mut child = Command::new("wg")
        .args(["set", &args.interface, "private-key", "/dev/stdin"])
        .stdin(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;
    stdin
        .write_all(private_key.into_base_64().as_bytes())
        .await?;
    stdin.flush().await?;
    let status = child.wait().await?;

    if !status.success() {
        return Err("Failed to set private key".into());
    }
    // Поднятие интерфейса wg0
    check(
        Command::new("ip")
            .args(["link", "set", "up", "dev", &args.interface])
            .status()
            .await?,
        "Failed to start interface",
    )?;

    Ok(())
}
async fn wireguard_add_peer(
    public_key: &wg::PublicKey,
    internal_address: IpNet,
    endpoint: &str,
    args: &Arguments,
) -> tonic::Result<()> {
    let pub_key: String = public_key.into_base_64();
    let status = tokio::process::Command::new("wg")
        .args([
            "set",
            &args.interface,
            "peer",
            &pub_key,
            "endpoint",
            endpoint,
            "allowed-ips",
            &internal_address.trunc().to_string(),
            "persistent-keepalive",
            &args.persistent_keepalive.to_string(),
        ])
        .status()
        .await
        .map_err(|err| tonic::Status::internal(format!("failed to run wg: {}", err)))?;

    if !status.success() {
        return Err(tonic::Status::internal(format!(
            "wg finished with {status}"
        )));
    }
    Ok(())
}

pub async fn execute(args: &Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint: String = (&args.host).into();
    let endpoint = TEndpoint::from_shared(endpoint)?;
    let keypair = KeyPair::gen();

    let mut client = DhcServiceClient::connect(endpoint).await?;
    let request = ReserveIpRequest {
        account: args.account.clone(),
        public_key: keypair.public.into_base_64(),
    };
    let response = client.reserve_ip(request).await?;
    let response = response.into_inner();

    setup_wireguard_interface(&keypair.private, &response.address, args).await?;
    wireguard_add_peer(
        &FromBase64::from_base_64(&response.server_public_key)?,
        response.address.parse()?,
        &response.endpoint,
        args,
    )
    .await?;

    Ok(())
}
