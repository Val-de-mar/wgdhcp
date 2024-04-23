use std::{process, io::Write};

use super::custom::{Key, check_key};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyPair {
    pub public: Key,
    pub private: Key,
}

pub fn generate_keypair() -> KeyPair {
    let private = process::Command::new("wg")
        .args(["genkey"])
        .output()
        .unwrap_or_else(|err| panic!("failed to execute generate private key: {}", err));

    let mut public_job = process::Command::new("wg")
        .args(["pubkey"])
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()
        .unwrap_or_else(|err| panic!("failed to execute generate public key: {}", err));
    
    if let Some(ref mut stdin) = public_job.stdin {
        stdin.write_all(&private.stdout).unwrap();
    }

    let public = public_job
        .wait_with_output()
        .unwrap_or_else(|err| panic!("failed to execute generate public key: {}", err));
    
    let mut public_key = Key(String::from_utf8(public.stdout).expect("failed to decode public key"));
    let mut private_key = Key(String::from_utf8(private.stdout).expect("failed to decode public key"));
    if public_key.0.chars().last().expect("empty generated public key") == '\n' {
        public_key.0.pop();
    }
    if private_key.0.chars().last().expect("empty generated private key") == '\n' {
        private_key.0.pop();
    }
    assert!(check_key(&public_key.0), "incorrect public key generated: {}", &public_key.0);
    assert!(check_key(&private_key.0), "incorrect public key generated: {}", &private_key.0);

    KeyPair{
        public: public_key,
        private: private_key,
    }
}