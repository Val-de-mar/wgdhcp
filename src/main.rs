#![feature(map_try_insert)]
#![feature(string_remove_matches)]

mod common;

mod client;

use std::error::Error;

use clap::{Parser, Subcommand};

pub mod commands;
pub mod service;

#[derive(Subcommand, Debug)]
enum Command {
    #[command(name = "init", about = "inits storage, has to be run before runserver")]
    Init,
    #[command(
        name = "runserver",
        about = "runs server with configuration from ~/.config/wgdhc.yaml"
    )]
    RunServer,
    #[command(name = "ls", about = "lists all profiles(works on server only)")]
    Ls,
    #[command(name = "client", about = "inits client wg peer")]
    Client(client::Arguments),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[command(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::parse();

    match args.command {
        Command::RunServer => {
            commands::run_server::execute().await?;
        }
        Command::Ls => {
            print!("{}", commands::ls::execute().await);
        }
        Command::Client(args) => {
            client::execute(&args).await?;
        }
        Command::Init => {
            commands::init::execute().await?;
        }
    };
    Ok(())
}
