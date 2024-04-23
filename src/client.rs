use std::{io::Write, process::{self, Stdio}};

use clap::Args;
use ipnet::IpNet;
use serde::{Serialize, Deserialize};

use crate::{common::{custom::{Key, Endpoint}, wg::generate_keypair}, storing, add};
use toml;
use crate::add::Response;

#[derive(Debug, Args)]
pub struct Arguments {
    pub ssh_host: Endpoint,

    #[arg(short, long, default_value_t = {"/etc/wireguard/wg0.conf".into()})]
    pub config: String,

    #[arg(short, long)]
    pub force: bool,

    #[arg(short, long, default_value_t = 20)]
    pub persistent_keepalive: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Interface {
    address: IpNet,
    private_key: Key,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Peer {
    public_key: Key,
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

fn get_response(endpoint: &Endpoint, public_key: Key) -> Response {
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

fn gen_client_config(args: &Arguments) -> ClientConfig {
    let pair = generate_keypair();
    let response = get_response(&args.ssh_host, pair.public);
    ClientConfig{
        interface: Interface {
            address: response.internal_addr.clone(),
            private_key: pair.private,
        },
        peer: Peer {
            public_key: response.server_pubkey,
            allowed_ip: response.internal_addr,
            endpoint: response.endpoint,
            persistent_keepalive: args.persistent_keepalive,
        }
    }
}

pub fn execute(args: &Arguments) {
    let config = gen_client_config(args);
    let config = toml::to_string(&config).unwrap_or_else(|err| panic!("cannot deserialize config: {}", err));
    let mut file = storing::open_file(&args.config, args.force);
    file.write_all(config.as_bytes()).unwrap()
}

