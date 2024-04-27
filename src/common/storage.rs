use std::{collections::HashMap, ops::{Deref, DerefMut}};

use serde::{Serialize, Deserialize};
use serde_with::serde_as;
use tokio::{fs::{self, OpenOptions}, io::{AsyncReadExt, AsyncWriteExt as _}};

use super::custom::Endpoint;

use std::net::IpAddr;
use ipnet::IpNet;
use derive_more::From;
use lazy_static::lazy_static;
use crate::common::{config::CONFIG, wg::{self, SerdeBase64}};

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct ServerInfo {
    #[serde_as(as = "SerdeBase64")]
    pub public_key: wg::PublicKey,
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

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Interface {
    pub listen_port: u16,
    #[serde_as(as = "SerdeBase64")]
    pub private_key: wg::PrivateKey,
    pub address: IpNet,
}

#[derive(Serialize, From)]
#[serde(rename_all = "PascalCase")]
pub struct WgConfigInterface<'a> {
    pub interface: &'a Interface
}

#[serde_as]
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PeerConfig<'a> {
    #[serde_as(as = "SerdeBase64")]
    pub public_key: &'a wg::PublicKey,
    #[serde(rename(serialize = "AllowedIPs"))]
    pub allowed_ips: &'a IpNet,
    pub endpoint: &'a Endpoint,
}

#[derive(Serialize, From)]
#[serde(rename_all = "PascalCase")]
pub struct WgConfigPeer<'a> {
    pub peer: &'a PeerConfig<'a>
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct Storage {
    pub interface: Interface,
    pub server: ServerInfo,
    #[serde_as(as = "HashMap<_, HashMap<SerdeBase64, _>>")]
    pub peers: HashMap<String, HashMap<wg::PublicKey, PeerInfo>>,
}

impl Storage {
    pub fn find_ip(&self) -> Option<IpAddr> {
        let mut used_ips: Vec<_> = self.peers.values().flatten().map(|x| x.1.internal_addr).collect();
        used_ips.push(self.interface.address.addr());
        let mut found = Option::<IpAddr>::None;
        for addr in self.interface.address.hosts() {
            if !used_ips.contains(&addr) {
                found = Some(addr);
                break;
            }
        }
        found
    }
    pub fn push(&mut self, account: &str, public_key: wg::PublicKey, peer: PeerInfo) -> PeerInfo {
        let _ = self.peers.try_insert(account.into(), Default::default());
        let peers_of_account = self.peers.get_mut(account).unwrap();
    
        match peers_of_account.try_insert(public_key, peer) {
            Ok(a) => a.clone(),
            Err(occupied) => occupied.entry.get().clone(),
        }
    }
}

pub struct StorageLock<'a> {
    storage: Option<Box<Storage>>,
    _lock: tokio::sync::MutexGuard<'a, ()>, 
}

impl<'a> Deref for StorageLock<'a> {
    type Target = Storage;

    fn deref(&self) -> &Self::Target {
        self.storage.as_deref().unwrap()
    }
}

impl<'a> DerefMut for StorageLock<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.storage.as_deref_mut().unwrap()
    }
}

lazy_static! {
    static ref STORAGE_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::new(()); 
}

#[derive(thiserror::Error, Debug)]
pub enum CommitError {
    #[error("io error: {}", .0)]
    IO(#[from] std::io::Error),
    #[error("serialize error: {}", .0)]
    Serialize(#[from] serde_yaml::Error),
}

pub async fn commit_storage(storage: &Storage) -> std::result::Result<(), CommitError> {
    // Создаём временный файл в той же директории, что и оригинальный файл, для сохранения fs
    let mut temp_path = CONFIG.storage.clone();
    temp_path.set_file_name(format!("{}.tmp.dump", temp_path.file_name().map(|x| x.to_str()).flatten().expect("cannot get storage filename")));

    let mut temp_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&temp_path)
        .await?;
    let result = serde_yaml::to_string(&storage)?;
    temp_file.write_all(result.as_bytes()).await?;
    temp_file.flush().await?;

    fs::rename(&temp_path, &CONFIG.storage).await;

    Ok(())
}

impl<'a> Drop for StorageLock<'a> {
    fn drop(&mut self) {
        let mut val = Option::<Box<Storage>>::None;
        std::mem::swap(&mut self.storage, &mut val);
        tokio::spawn(async move {
            commit_storage(val.as_deref().unwrap()).await.expect("cannot commit_storage");
        });
    }
}

pub async fn get_storage() -> StorageLock<'static> {
    let lock: tokio::sync::MutexGuard<'static, ()> = STORAGE_MUTEX.lock().await;
    let mut file = tokio::fs::File::open(&CONFIG.storage).await.unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).await.expect("cannot read storage");

    StorageLock{
        storage: Some(Box::new(serde_yaml::from_str(&string).unwrap())),
        _lock: lock
    }
}

pub fn get_interface_config(storage: &Storage) -> String {
    toml::to_string(&WgConfigInterface::from(&storage.interface)).unwrap()
}

pub fn get_peer_config(public_key: &wg::PublicKey, allowed_ips: &IpNet, endpoint: &Endpoint) -> String {
    toml::to_string(&WgConfigPeer{
        peer: &PeerConfig{
            public_key: public_key,
            allowed_ips: allowed_ips,
            endpoint: endpoint,
        },
    }).unwrap()
}