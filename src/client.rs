use std::{io::Write, process::{self, Stdio}};

use clap::Args;
use ipnet::IpNet;
use serde::{Serialize, Deserialize};

use crate::{add, common::{custom::Endpoint, wg::{self, IntoBase64, KeyPair}}};
use crate::add::Response;
use tonic::transport::channel::Endpoint as TEndpoint;

use crate::common::proto::{
    dhc_service_client::DhcServiceClient,
    ReserveIpRequest, ReserveIpResponse
};

#[derive(Debug, Args)]
pub struct Arguments {
    #[clap(help="wg dhc server address")]
    pub host: Endpoint,
    pub account: String,
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

fn get_response(endpoint: &Endpoint, public_key: wg::PublicKey) -> Response {
    let mut args: Vec<String> = Default::default();
    args.push(format!("getaccess@{}", &endpoint.host));
    if let Some(port) = endpoint.port {
        args.push("-p".into());
        args.push(format!("{port}"))
    }
    let mut job = process::Command::new("ssh")
        .args(args)
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .unwrap_or_else(|err| panic!("failed to run ssh: {}", err));

    let query = add::Query{
        public_key: public_key
    };

    let query = serde_yaml::to_string(&query).unwrap();

    if let Some(ref mut stdin) = job.stdin {
        stdin.write_all(query.as_bytes()).unwrap();
    }
    let out = job.wait_with_output(). unwrap_or_else(|err| panic!("failed to run ssh: {}", err));

    assert!(out.status.success(), "ssh failed with exit status: {}\n stderr:\n{:?}", &out.status, unsafe{String::from_utf8_unchecked(out.stderr)});
    let result = String::from_utf8(out.stdout).expect("cannot read output of ssh request");
    serde_yaml::from_str(&result).expect("error parsing server response")
}

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
    let request = ReserveIpRequest { account: args.account.clone(), public_key: keypair.public.into_base_64()};
    let response = client.reserve_ip(request).await?;
    let response = response.into_inner();

    println!(
"
[Interface]\n
[Peer]\nPublicKey = {}\nAllowedIPs = {}\nEndpoint = {}\n
",
    response.server_public_key,
    response.address,
    response.
    )
    
    Ok(())
}

