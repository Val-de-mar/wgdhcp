use clap::Args;
use ipnet::IpNet;

use crate::common::storage::{get_interface_config, get_peer_config};
use crate::storing::*;

#[derive(Args, Debug)]
pub struct Arguments {
    #[arg(short, long, default_value_t = {"/data/storage.yaml".into()})]
    pub storage: String,
}

pub fn execute(args: &Arguments) -> String {
    let storage = read_storage_from_file(&args.storage);
    let interface_config = get_interface_config(&storage);
    let mut result = interface_config;
    for (_account, peers)  in &storage.peers {
        for (key, info) in peers {
            let allowed_ips = IpNet::new(info.internal_addr.clone(), 0).unwrap();
            let allowed_ips = IpNet::new(info.internal_addr.clone(), allowed_ips.max_prefix_len()).unwrap();
            result += &get_peer_config(key, &allowed_ips, &storage.server.endpoint);
        }
    }
    result.remove_matches("\"");
    result
}