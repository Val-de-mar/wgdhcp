use serde::{Serialize, Deserialize};
use clap::Args;
use crate::common::{custom::{Key, Endpoint}, storage::*, wg::generate_keypair};
use crate::storing::*;
use ipnet::IpNet;

#[derive(Args, Debug)]
pub struct Arguments {
    #[arg(short, long, default_value_t = {"/data/storage.yaml".into()})]
    pub storage: String,

    #[arg(short, long, default_value_t = {"/etc/wireguard/wg0.conf".into()})]
    pub config: String,

    #[arg(short, long)]
    pub force: bool,

    #[arg(short, long, help = "regenerates key if set")]
    pub regenerate: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Query {
    pub endpoint: Endpoint,
    pub listen_port: u16,
    pub public_key: Key,
    pub private_key: Key,
    pub address: IpNet,
    pub save_config: bool,
}


impl From<&Query> for ServerInfo {
    fn from(value: &Query) -> Self {
        ServerInfo {
            public_key: value.public_key.clone(),
            endpoint: value.endpoint.clone(),
        }
    }
}

impl From<&Query> for Interface {
    fn from(value: &Query) -> Self {
        Interface{
            listen_port: value.listen_port,
            private_key: value.private_key.clone(),
            address: value.address.clone(),
            save_config: value.save_config,
        }
    }
}

impl From<&Query> for Storage {
    fn from(value: &Query) -> Self {
        Storage {
            interface: Interface::from(value),
            server: From::from(value),
            peers: Default::default(),
        }
    }
}

fn regenerate_keys(storage: &mut Storage) {
    let keypair = generate_keypair();
    storage.interface.private_key = keypair.private;
    storage.server.public_key = keypair.public;
}

pub fn execute(args: &Arguments, query: &Query) {
    let mut storage = Storage::from(query);
    if args.regenerate {
        regenerate_keys(&mut storage);
    }
    commit_storage(&args.storage, &storage, args.force);
    init_config(&args.config, &storage, args.force);
}
