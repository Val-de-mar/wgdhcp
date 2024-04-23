use std::{collections::HashMap, ops::{Deref, DerefMut}};

use serde::{Serialize, Deserialize};
use tokio::io::AsyncReadExt;

use super::custom::{Key, Endpoint};

use std::net::IpAddr;
use ipnet::IpNet;
use derive_more::From;
use lazy_static::lazy_static;
use crate::common::config::CONFIG;


#[derive(Serialize, Deserialize)]
pub struct ServerInfo {
    pub public_key: Key,
    pub endpoint: Endpoint,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PeerInfo {
    pub internal_addr: IpAddr,
}

impl From<IpAddr> for PeerInfo {
    fn from(value: IpAddr) -> Self {
        PeerInfo {
            internal_addr: value,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Interface {
    pub listen_port: u16,
    pub private_key: Key,
    pub address: IpNet,
    pub save_config: bool,
}

#[derive(Serialize, From)]
#[serde(rename_all = "PascalCase")]
pub struct WgConfigInterface<'a> {
    pub interface: &'a Interface
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PeerConfig<'a> {
    pub public_key: &'a Key,
    #[serde(rename(serialize = "AllowedIPs"))]
    pub allowed_ips: &'a IpNet,
    pub endpoint: &'a Endpoint,
}

#[derive(Serialize, From)]
#[serde(rename_all = "PascalCase")]
pub struct WgConfigPeer<'a> {
    pub peer: &'a PeerConfig<'a>
}

#[derive(Serialize, Deserialize)]
pub struct Storage {
    pub interface: Interface,
    pub server: ServerInfo,
    pub peers: HashMap<String, HashMap<Key, PeerInfo>>,
}

impl Storage {
    pub fn find_ip(&self) -> Option<IpAddr> {
        let used_ips: Vec<_> = self.peers.values().flatten().map(|x| x.1.internal_addr).collect();
        let mut found = Option::<IpAddr>::None;
        for addr in self.interface.address.hosts() {
            if !used_ips.contains(&addr) {
                found = Some(addr);
                break;
            }
        }
        found
    }
    pub fn push(&mut self, account: &str, public_key: Key, peer: PeerInfo) -> PeerInfo {
        let _ = self.peers.try_insert(account.into(), Default::default());
        let peers_of_account = self.peers.get_mut(account).unwrap();
    
        match peers_of_account.try_insert(public_key, peer) {
            Ok(a) => a.clone(),
            Err(occupied) => occupied.entry.get().clone(),
        }
    }
}

pub struct StorageLock<'a> {
    storage: Storage,
    lock: tokio::sync::MutexGuard<'a, ()>, 
}

impl<'a> Deref for StorageLock<'a> {
    type Target = Storage;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

impl<'a> DerefMut for StorageLock<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.storage
    }
}

lazy_static! {
    static ref STORAGE_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::new(()); 
}

pub async fn get_storage() -> StorageLock<'static> {
    let lock: tokio::sync::MutexGuard<'static, ()> = STORAGE_MUTEX.lock().await;
    let mut file = tokio::fs::File::open(&CONFIG.storage).await.unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).await.expect("cannot read storage");

    StorageLock{
        storage: serde_yaml::from_str(&string).unwrap(),
        lock
    }
}

pub fn get_interface_config(storage: &Storage) -> String {
    toml::to_string(&WgConfigInterface::from(&storage.interface)).unwrap()
}

pub fn get_peer_config(public_key: &Key, allowed_ips: &IpNet, endpoint: &Endpoint) -> String {
    toml::to_string(&WgConfigPeer{
        peer: &PeerConfig{
            public_key: public_key,
            allowed_ips: allowed_ips,
            endpoint: endpoint,
        },
    }).unwrap()
}