use lazy_static::lazy_static;
use serde::Deserialize;
use crate::common::custom::Endpoint;
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Deserialize, Clone, Debug)]
pub struct RepoInstance {
    pub mountpoint: PathBuf,
    pub ide: PathBuf,
    pub profile: Option<String>,
}

fn default_addr() -> IpAddr {
    "0.0.0.0".parse().unwrap()
}
fn default_port() -> u16 {
    5010
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
}

fn get_config() -> Config {
    let config = shellexpand::tilde("~/.config/wgdhcp.yaml").to_string();
    let file = std::fs::File::open(&config).unwrap();
    serde_yaml::from_reader(&file)
        .map_err(|error| panic!("cannot read config file with error {}", error))
        .unwrap()
}

lazy_static! {
    pub static ref CONFIG: Config = get_config();
}
