use std::collections::HashMap;

use crate::common::{config::CONFIG, storage::*, wg::KeyPair};

pub async fn execute() -> std::result::Result<(), CommitError> {
    let keypair = KeyPair::gen();
    let storage = Storage {
        interface: Interface {
            listen_port: CONFIG.service.port.clone(),
            private_key: keypair.private,
            address: CONFIG.internal_address.clone(),
        },
        server: ServerInfo {
            public_key: keypair.public,
            endpoint: CONFIG.service.endpoint.clone(),
        },
        peers: HashMap::default(),
    };
    commit_storage(&storage).await
}
