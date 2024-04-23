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

use clap::{Parser, Subcommand};
use regex::Regex;
use serde_yaml;


#[derive(Subcommand, Debug)]
enum Command {
    #[command(name="add-peer", about="adds peer to configuration and returns information for client")]
    Add(add::Arguments),
    #[command(name="init", about="inits config file and storage")]
    Init(init::Arguments),
    #[command(name="add-key", about="adds public key to the account, key is to be delivered by stdin")]
    AddKey(add_key::Arguments),
    #[command(name="ls")]
    Ls(ls::Arguments),
    #[command(name="start", about="starts wireguard service")]
    Start(start::Arguments),
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

fn main() {

    let args = Arguments::parse();

    match args.command {
        Command::Add(args) => {
            let query: add::Query = serde_yaml::from_reader(std::io::stdin()).unwrap();
            let response = add::execute(&args, &query);
            serde_yaml::to_writer(std::io::stdout(), &response).expect("cannot write to stdout");
        },
        Command::Init(args) => {
            let query: init::Query = serde_yaml::from_reader(std::io::stdin()).unwrap();
            init::execute(&args, &query);
        },
        Command::AddKey(args) => {
            validate_account_name(&args.account);
            let ssh_public_key = std::io::read_to_string(std::io::stdin()).expect("cannot read public_key");
            add_key::execute(&args, &ssh_public_key);
        },
        Command::Ls(args) => {
            print!("{}", ls::execute(&args));
        },
        Command::Start(args) => {
            start::execute(&args);
        },
        Command::Client(args) => {
            client::execute(&args);
        },
        Command::Gencofig(args) => {
            print!("{}", genconfig::execute(&args));
        },
    };
}
