use serde_yaml;
use std::{fs::{self, File}, io::Write};
use crate::common::storage::*;
use std::io::ErrorKind;


pub fn read_storage_from_file(path: &str) -> Storage {
    let file = fs::File::open(path).unwrap();
    serde_yaml::from_reader(file).unwrap()
}

pub fn commit_storage(path: &str, storage: &Storage, force: bool) {
    let output = 
        if force {
            fs::File::create(path)
        } else {
            fs::File::create_new(path)
        }.unwrap_or_else(process_openning_error);
    serde_yaml::to_writer(&output, storage).unwrap();
}


fn process_openning_error<T>(err: std::io::Error) -> T{
    if err.kind() == ErrorKind::AlreadyExists {
        panic!("config file exists, you should run command with '--force' to override it")
    } else {
        panic!("error while openning config: {}", err);
    }
}

pub fn open_file(path: &str, force: bool) -> File {
    if force {
        fs::File::create(path)
    } else {
        fs::File::create_new(path)
    }.unwrap_or_else(process_openning_error)
}

pub fn init_config(path: &str, storage: &Storage, force: bool) {
    let mut config = get_interface_config(&storage);
    config.remove_matches("\"");
    let mut output = open_file(path, force);

    output.write_all(config.as_bytes()).unwrap();
}