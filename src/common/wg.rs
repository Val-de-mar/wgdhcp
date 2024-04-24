use std::str::FromStr;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use derive_more as dm;
use rand::rngs::OsRng;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use x25519_dalek::{PublicKey, StaticSecret as PrivateKey};

#[derive(Clone)]
pub struct KeyPair {
    pub public: PublicKeyBuf,
    pub private: PrivateKeyBuf,
}

impl KeyPair {
    pub fn gen() -> KeyPair {
        let private_key: PrivateKeyBuf = PrivateKey::random_from_rng(OsRng).into();

        let public_key: PublicKeyBuf = PublicKey::from(&private_key.key).into();

        KeyPair {
            public: public_key,
            private: private_key,
        }
    }
}

#[derive(dm::Into, dm::From, Clone)]
pub struct PrivateKeyBuf {
    key: PrivateKey,
}

impl From<&PrivateKeyBuf> for String {
    fn from(value: &PrivateKeyBuf) -> Self {
        STANDARD.encode(value.key.as_bytes())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("cannot decode base64: {}", .0)]
    NotBase64(String),
    #[error("incorrect key length after base64 decode: {}", .0)]
    IncorrectLength(String),
}

impl FromStr for PrivateKeyBuf {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes_vec = STANDARD
            .decode(s)
            .map_err(|_| ParseError::NotBase64(s.to_string()))?;
        let bytes: &[u8; 32] = bytes_vec
            .as_slice()
            .try_into()
            .map_err(|_| ParseError::IncorrectLength(s.to_string()))?;
        Ok(Self{key: bytes.clone().into()})
    }
}

impl Serialize for PrivateKeyBuf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s: String = self.into();
        s.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PrivateKeyBuf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        FromStr::from_str(&string).map_err(|e|D::Error::custom(format!("{e}")))
    }
}

#[derive(dm::Into, dm::From, Clone, PartialEq, Eq, Hash)]
pub struct PublicKeyBuf {
    key: PublicKey,
}

impl From<&PublicKeyBuf> for String {
    fn from(value: &PublicKeyBuf) -> Self {
        STANDARD.encode(value.key.as_bytes())
    }
}

impl FromStr for PublicKeyBuf {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes_vec = STANDARD
            .decode(s)
            .map_err(|_| ParseError::NotBase64(s.to_string()))?;
        let bytes: &[u8; 32] = bytes_vec
            .as_slice()
            .try_into()
            .map_err(|_| ParseError::IncorrectLength(s.to_string()))?;
        Ok(Self{key: bytes.clone().into()})
    }
}

impl Serialize for PublicKeyBuf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s: String = self.into();
        s.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PublicKeyBuf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        FromStr::from_str(&string).map_err(|e|D::Error::custom(format!("{e}")))
    }
}

// pub async fn generate_keypair() -> KeyPair {
//     let private = process::Command::new("wg")
//         .args(["genkey"])
//         .output()
//         .await
//         .unwrap_or_else(|err| panic!("failed to execute generate private key: {}", err));

//     let mut public_job = process::Command::new("wg")
//         .args(["pubkey"])
//         .stdin(std::process::Stdio::piped())
//         .stdout(std::process::Stdio::piped())
//         .spawn()
//         .unwrap_or_else(|err| panic!("failed to execute generate public key: {}", err));

//     if let Some(ref mut stdin) = public_job.stdin {
//         stdin.write_all(&private.stdout).await.unwrap();
//     }

//     let public = public_job
//         .wait_with_output()
//         .await
//         .unwrap_or_else(|err| panic!("failed to execute generate public key: {}", err));

//     let mut public_key = Key(String::from_utf8(public.stdout).expect("failed to decode public key"));
//     let mut private_key = Key(String::from_utf8(private.stdout).expect("failed to decode public key"));
//     if public_key.0.chars().last().expect("empty generated public key") == '\n' {
//         public_key.0.pop();
//     }
//     if private_key.0.chars().last().expect("empty generated private key") == '\n' {
//         private_key.0.pop();
//     }
//     assert!(check_key(&public_key.0), "incorrect public key generated: {}", &public_key.0);
//     assert!(check_key(&private_key.0), "incorrect public key generated: {}", &private_key.0);

//     KeyPair{
//         public: public_key,
//         private: private_key,
//     }
// }
