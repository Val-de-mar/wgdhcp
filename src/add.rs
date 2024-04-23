use serde::{Serialize, Deserialize};
use clap::Args;
use crate::common::{custom::{Key, Endpoint}, storage::*};
use crate::storing::*;
use ipnet::IpNet;
use std::process;

#[derive(Args, Debug)]
pub struct Arguments {
    pub account: String,

    #[arg(short, long, default_value_t = {"/data/storage.yaml".into()})]
    pub storage: String,

    #[arg(long, help="adds peer to wireguard if set(user should have permissions to do that)")]
    pub wg: bool,

    #[arg(short, long, required_if_eq("wg", "true"))]
    pub interface: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Query {
    pub public_key: Key,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub internal_addr: IpNet,
    pub server_pubkey: Key,
    pub endpoint: Endpoint,
}

fn wireguard_add_peer(args: &Arguments, public_key: &Key, info: &PeerInfo) {
    let allowed_ips = IpNet::new(info.internal_addr.clone(), 0).unwrap();
    let allowed_ips = IpNet::new(info.internal_addr.clone(), allowed_ips.max_prefix_len()).unwrap();

    let status = process::Command::new("sudo")
        .args([
            "wg",
            "set", args.interface.as_ref().unwrap(),
            "peer", &public_key.0,
            "allowed-ips", &allowed_ips.to_string()])
        .status()
        .expect("failed to execute wg");
    
    assert!(status.success(), "wg finished with: {}", status);
}

pub fn execute(args: &Arguments, query: &Query) -> Response {
    let mut storage: Storage = read_storage_from_file(&args.storage);

    let ip = storage.find_ip().expect("all ip addresses in subnetwork are used");

    let new_peer = PeerInfo{
        internal_addr: ip.clone(),
    };

    let new_peer = storage.push(&args.account, query.public_key.clone(), new_peer.clone());
    if args.wg {
        wireguard_add_peer(&args, &query.public_key, &new_peer);
    }
    commit_storage(&args.storage, &storage, true);

    let internal_addr = IpNet::new(
        new_peer.internal_addr,
        storage.interface.address.prefix_len()
    ).expect("cannot create new address with mask from address and mask");

    Response{
        internal_addr: internal_addr,
        server_pubkey: storage.server.public_key,
        endpoint: storage.server.endpoint,
    }
}