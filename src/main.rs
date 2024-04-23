#![feature(file_create_new)]
#![feature(map_try_insert)]
#![feature(string_remove_matches)]

mod common;
mod storing;

// modules of commands
mod init;
mod add;
mod add_key;
mod ls;
mod start;
mod client;
mod genconfig;

use std::{error::Error, net::IpAddr, net::SocketAddr, str::FromStr};

use clap::{Parser, Subcommand};
use common::config::CONFIG;
use regex::Regex;
use serde_yaml;
use tonic::transport::Server;


pub mod service;

// #[derive(Subcommand, Debug)]
// enum Command {
//     #[command(name="add-peer", about="adds peer to configuration and returns information for client")]
//     Add(add::Arguments),
//     #[command(name="init", about="inits config file and storage")]
//     Init(init::Arguments),
//     #[command(name="add-key", about="adds public key to the account, key is to be delivered by stdin")]
//     AddKey(add_key::Arguments),
//     #[command(name="ls")]
//     Ls(ls::Arguments),
//     #[command(name="start", about="starts wireguard service")]
//     Start(start::Arguments),
//     #[command(name="genconfig", about="generate wireguard configuration file for server")]
//     Gencofig(genconfig::Arguments),
//     #[command(name="client", about="sets up client config file")]
//     Client(client::Arguments),
// }

#[derive(Subcommand, Debug)]
enum Command {
    #[command(name="runserver", about="runs server with configuration from ~/.config/wgdhcp.yaml")]
    RunServer,
    #[command(name="ls")]
    Ls(ls::Arguments),
    #[command(name="genconfig", about="generate wireguard configuration file for server")]
    Gencofig(genconfig::Arguments),
    #[command(name="client", about="sets up client config file")]
    Client(client::Arguments),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[command(subcommand)]
    command: Command,
}


fn validate_account_name(account: &str) {
    let regex = Regex::new(r"^[0-9a-zA-Z]?{3,32}$").unwrap();
    assert!(regex.is_match(account), "account must be string of digits and letters at least 5 at most 32 characters");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{

    let args = Arguments::parse();

    match args.command {
        Command::RunServer => {
            let addr = SocketAddr::new(CONFIG.service.address, CONFIG.service.port);
            let service = service::ServiceImpl{};
            Server::builder()
                .add_service(service::DhcServiceServer::new(service))
                .serve(addr)
                .await?;
        },
        Command::Ls(args) => {
            print!("{}", ls::execute(&args));
        },
        Command::Client(args) => {
            client::execute(&args);
        },
        Command::Gencofig(args) => {
            print!("{}", genconfig::execute(&args));
        },
    };
    Ok(())
}
