use std::{
    io::Write,
    net::IpAddr,
    process::{self, ExitStatus, Stdio},
};

use clap::Args;
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncWriteExt, process::Command};

use crate::{
    add,
    common::{
        custom::Endpoint,
        wg::{self, IntoBase64, KeyPair, PrivateKey, PublicKey},
    },
};
use crate::common::wg::FromBase64;
use tonic::transport::channel::Endpoint as TEndpoint;

use crate::common::proto::{
    dhc_service_client::DhcServiceClient, ReserveIpRequest, ReserveIpResponse,
};

#[derive(Debug, Args)]
pub struct Arguments {
    #[clap(help = "wg dhc server address")]
    pub host: String,
    pub account: String,
    #[clap(default_value_t={"wg0".to_string()})]
    pub interface: String,
    #[clap(default_value_t={5})]
    pub persistent_keepalive: usize,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Interface {
    address: IpNet,
    private_key: wg::PrivateKey,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Peer {
    public_key: wg::PublicKey,
    allowed_ip: IpNet,
    endpoint: Endpoint,
    persistent_keepalive: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ClientConfig {
    interface: Interface,
    peer: Peer,
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
    check(Command::new("ip")
        .args(["link", "add", &args.interface, "type", "wireguard"])
        .status()
        .await?, "Failed to add interface")?;

    // Назначение IP адреса интерфейсу wg0
    check(Command::new("ip")
        .args(["address", "add", internal_address, "dev", &args.interface])
        .status()
        .await?, "Failed to set ip to interface")?;

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
    check(Command::new("ip")
        .args(["link", "set", "up", "dev", &args.interface])
        .status()
        .await?, "Failed to start interface")?;



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
// fn get_response(endpoint: &Endpoint, public_key: wg::PublicKey) -> Response {
//     let mut args: Vec<String> = Default::default();
//     args.push(format!("getaccess@{}", &endpoint.host));
//     if let Some(port) = endpoint.port {
//         args.push("-p".into());
//         args.push(format!("{port}"))
//     }
//     let mut job = process::Command::new("ssh")
//         .args(args)
//         .stdout(Stdio::piped())
//         .stdin(Stdio::piped())
//         .spawn()
//         .unwrap_or_else(|err| panic!("failed to run ssh: {}", err));

//     let query = add::Query {
//         public_key: public_key,
//     };

//     let query = serde_yaml::to_string(&query).unwrap();

//     if let Some(ref mut stdin) = job.stdin {
//         stdin.write_all(query.as_bytes()).unwrap();
//     }
//     let out = job
//         .wait_with_output()
//         .unwrap_or_else(|err| panic!("failed to run ssh: {}", err));

//     assert!(
//         out.status.success(),
//         "ssh failed with exit status: {}\n stderr:\n{:?}",
//         &out.status,
//         unsafe { String::from_utf8_unchecked(out.stderr) }
//     );
//     let result = String::from_utf8(out.stdout).expect("cannot read output of ssh request");
//     serde_yaml::from_str(&result).expect("error parsing server response")
// }

// fn gen_client_config(args: &Arguments) -> ClientConfig {
//     let pair = KeyPair::gen();
//     let response = get_response(&args.ssh_host, pair.public);
//     ClientConfig{
//         interface: Interface {
//             address: response.internal_addr.clone(),
//             private_key: pair.private,
//         },
//         peer: Peer {
//             public_key: response.server_pubkey,
//             allowed_ip: response.internal_addr,
//             endpoint: response.endpoint,
//             persistent_keepalive: args.persistent_keepalive,
//         }
//     }
// }

pub async fn execute(args: &Arguments) -> Result<(), Box<dyn std::error::Error>> {
    // let config = gen_client_config(args);
    // let config = toml::to_string(&config).unwrap_or_else(|err| panic!("cannot deserialize config: {}", err));
    // let mut file = storing::open_file(&args.config, args.force);
    // file.write_all(config.as_bytes()).unwrap()
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
