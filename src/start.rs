use clap::Args;
use std::process;

#[derive(Args, Debug)]
pub struct Arguments {
    #[arg(short, long, default_value_t = {"/etc/wireguard/wg0.conf".into()})]
    pub config: String,
}

pub fn execute(args: &Arguments) {
    let status = process::Command::new("wg-quick")
        .args(["up", &args.config])
        .status()
        .expect("failed to execute wg");
    assert!(status.success(), "wg-quick finished with: {}", status);
}
