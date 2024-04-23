
use clap::Args;
use std::{io::Write, fs::OpenOptions};

#[derive(Args, Debug)]
pub struct Arguments {
    pub account: String,

    #[arg(short, long, default_value_t = {"/home/getaccess/.ssh/authorized_keys".into()})]
    pub authorized_keys: String,
}

pub fn execute(args: &Arguments, ssh_public_key: &str) {
    let output = format!("environment=\"ACCOUNT={}\" {}\n", args.account, ssh_public_key);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&args.authorized_keys)
        .unwrap();

    if let Err(e) = write!(file, "{}", output) {
        panic!("Couldn't write to authorized_keys\n\tfile: {}\n\terror{}", &args.authorized_keys, e);
    }
}

