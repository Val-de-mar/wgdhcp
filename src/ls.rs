use crate::storing::*;
use clap::Args;

#[derive(Args, Debug)]
pub struct Arguments {
    #[arg(short, long, default_value_t = {"/data/storage.yaml".into()})]
    pub storage: String,
}


pub fn execute(args: &Arguments) -> String {
    let storage = read_storage_from_file(&args.storage);
    serde_yaml::to_string(&storage.peers).unwrap()
}