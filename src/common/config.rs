use crate::common::custom::Endpoint;
use ipnet::IpNet;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::net::IpAddr;
use std::path::PathBuf;

fn default_addr() -> IpAddr {
    "0.0.0.0".parse().unwrap()
}
fn default_internal_addr() -> IpNet {
    "10.11.0.1/16".parse().unwrap()
}
fn default_port() -> u16 {
    5010
}
fn default_wireguard_port() -> u16 {
    55000
}

#[derive(Deserialize, Clone, Debug)]
pub struct Service {
    #[serde(default = "default_addr")]
    pub address: IpAddr,
    #[serde(default = "default_port")]
    pub port: u16,
    pub endpoint: Endpoint,
}

#[derive(Deserialize)]
pub struct Config {
    pub service: Service,
    pub storage: PathBuf,
    pub interface: String,
    #[serde(default = "default_internal_addr")]
    pub internal_address: IpNet,
    #[serde(default = "default_wireguard_port")]
    pub wgport: u16,
}

fn get_config() -> Config {
    let config = shellexpand::tilde("~/.config/wgdhc.yaml").to_string();
    let file = std::fs::File::open(&config).unwrap();
    serde_yaml::from_reader(&file)
        .map_err(|error| panic!("cannot read config file with error {}", error))
        .unwrap()
}

lazy_static! {
    pub static ref CONFIG: Config = get_config();
}
