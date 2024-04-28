pub use crate::common::proto::{
    dhc_service_server::{DhcService, DhcServiceServer},
    ReserveIpRequest, ReserveIpResponse,
};
use ipnet::IpNet;
use tonic::Response;

use crate::common::{
    config::CONFIG,
    storage::{self, PeerInfo},
    wg::{self, FromBase64, IntoBase64 as _, PublicKey},
};

pub struct ServiceImpl {}

async fn wireguard_add_peer(public_key: &wg::PublicKey, info: &PeerInfo) -> tonic::Result<()> {
    let allowed_ips = IpNet::new(info.internal_addr.clone(), 0).unwrap();
    let allowed_ips = IpNet::new(info.internal_addr.clone(), allowed_ips.max_prefix_len()).unwrap();
    let pub_key: String = public_key.into_base_64();
    let status = tokio::process::Command::new("wg")
        .args([
            "set",
            &CONFIG.interface,
            "peer",
            &pub_key,
            "allowed-ips",
            &allowed_ips.to_string(),
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

#[tonic::async_trait]
impl DhcService for ServiceImpl {
    async fn reserve_ip(
        &self,
        request: tonic::Request<ReserveIpRequest>,
    ) -> tonic::Result<tonic::Response<ReserveIpResponse>> {
        let ans = {
            let mut storage = storage::get_storage().await;
            let req = request.get_ref();
            let ip = storage.find_ip().ok_or(tonic::Status::resource_exhausted(
                "all ip addresses are in use",
            ))?;
            let new_peer = PeerInfo {
                internal_addr: ip.clone(),
            };
            let public_key: PublicKey = FromBase64::from_base_64(&req.public_key).map_err(|e| {
                tonic::Status::invalid_argument(format!("incorrect public key: {e}"))
            })?;
            let new_peer = storage.push(&req.account, public_key.clone(), new_peer.clone());

            wireguard_add_peer(&public_key, &new_peer).await?;

            let internal_addr = IpNet::new(
                new_peer.internal_addr,
                storage.interface.address.prefix_len(),
            )
            .expect("cannot create new address with mask from address and mask");

            ReserveIpResponse {
                address: internal_addr.to_string(),
                server_public_key: storage.server.public_key.into_base_64(),
                endpoint: (&storage.server.endpoint).into(),
            }
        };
        Ok(Response::new(ans))
    }
}
