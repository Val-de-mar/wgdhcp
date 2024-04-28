use crate::common::storage::get_storage;

pub async fn execute() -> String {
    let storage = get_storage().await;
    serde_yaml::to_string(&storage.peers).unwrap()
}
