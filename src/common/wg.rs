use base64::{engine::general_purpose::STANDARD, Engine as _};
use rand::rngs::OsRng;
use serde::{de::Error as _, Deserialize, Deserializer, Serializer};
pub use x25519_dalek::{PublicKey, StaticSecret as PrivateKey};

use serde_with::{DeserializeAs, SerializeAs};

#[derive(Clone)]
pub struct KeyPair {
    pub public: PublicKey,
    pub private: PrivateKey,
}

impl KeyPair {
    pub fn gen() -> KeyPair {
        let private_key: PrivateKey = PrivateKey::random_from_rng(OsRng);

        let public_key: PublicKey = PublicKey::from(&private_key);

        KeyPair {
            public: public_key,
            private: private_key,
        }
    }
}

pub trait FromBase64 {
    fn from_base_64(value: &str) -> Result<Self, ParseError>
    where
        Self: Sized;
}

impl FromBase64 for PublicKey {
    fn from_base_64(value: &str) -> Result<Self, ParseError> {
        let bytes_vec = STANDARD
            .decode(value)
            .map_err(|_| ParseError::NotBase64(value.to_string()))?;
        let bytes: &[u8; 32] = bytes_vec
            .as_slice()
            .try_into()
            .map_err(|_| ParseError::IncorrectLength(value.to_string()))?;
        Ok(bytes.clone().into())
    }
}

impl FromBase64 for PrivateKey {
    fn from_base_64(value: &str) -> Result<Self, ParseError> {
        let bytes_vec = STANDARD
            .decode(value)
            .map_err(|_| ParseError::NotBase64(value.to_string()))?;
        let bytes: &[u8; 32] = bytes_vec
            .as_slice()
            .try_into()
            .map_err(|_| ParseError::IncorrectLength(value.to_string()))?;
        Ok(bytes.clone().into())
    }
}

pub trait IntoBase64 {
    fn into_base_64(&self) -> String;
}

impl IntoBase64 for PublicKey {
    fn into_base_64(&self) -> String {
        STANDARD.encode(self.as_bytes())
    }
}

impl IntoBase64 for PrivateKey {
    fn into_base_64(&self) -> String {
        STANDARD.encode(self.as_bytes())
    }
}

pub struct SerdeBase64 {}

impl SerdeBase64 {
    pub fn encode<const N: usize, T: for<'a> Into<&'a [u8; N]>>(t: T) -> String {
        STANDARD.encode(t.into())
    }
}

impl SerializeAs<PublicKey> for SerdeBase64 {
    fn serialize_as<S>(source: &PublicKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&STANDARD.encode(&source.as_bytes()))
    }
}

impl SerializeAs<PrivateKey> for SerdeBase64 {
    fn serialize_as<S>(source: &PrivateKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&STANDARD.encode(&source.as_bytes()))
    }
}

impl SerializeAs<&PublicKey> for SerdeBase64 {
    fn serialize_as<S>(source: &&PublicKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&STANDARD.encode(&(*source).as_bytes()))
    }
}

impl SerializeAs<&PrivateKey> for SerdeBase64 {
    fn serialize_as<S>(source: &&PrivateKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&STANDARD.encode(&(*source).as_bytes()))
    }
}

impl<'de> DeserializeAs<'de, PublicKey> for SerdeBase64 {
    fn deserialize_as<D>(deserializer: D) -> Result<PublicKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        FromBase64::from_base_64(&string).map_err(|e| D::Error::custom(format!("{e}")))
    }
}

impl<'de> DeserializeAs<'de, PrivateKey> for SerdeBase64 {
    fn deserialize_as<D>(deserializer: D) -> Result<PrivateKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        FromBase64::from_base_64(&string).map_err(|e| D::Error::custom(format!("{e}")))
    }
}

// #[derive(dm::Into, dm::From, Clone)]
// pub struct PrivateKeyBuf {
//     key: PrivateKey,
// }

// impl From<&PrivateKeyBuf> for String {
//     fn from(value: &PrivateKeyBuf) -> Self {
//         STANDARD.encode(value.key.as_bytes())
//     }
// }

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("cannot decode base64: {}", .0)]
    NotBase64(String),
    #[error("incorrect key length after base64 decode: {}", .0)]
    IncorrectLength(String),
}

// impl FromStr for PrivateKeyBuf {
//     type Err = ParseError;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let bytes_vec = STANDARD
//             .decode(s)
//             .map_err(|_| ParseError::NotBase64(s.to_string()))?;
//         let bytes: &[u8; 32] = bytes_vec
//             .as_slice()
//             .try_into()
//             .map_err(|_| ParseError::IncorrectLength(s.to_string()))?;
//         Ok(Self {
//             key: bytes.clone().into(),
//         })
//     }
// }

// impl Serialize for PrivateKeyBuf {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let s: String = self.into();
//         s.serialize(serializer)
//     }
// }

// impl<'de> Deserialize<'de> for PrivateKeyBuf {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let string = String::deserialize(deserializer)?;
//         FromStr::from_str(&string).map_err(|e|D::Error::custom(format!("{e}")))
//     }
// }

// impl From<&PublicKeyBuf> for String {
//     fn from(value: &PublicKeyBuf) -> Self {
//         STANDARD.encode(value.key.as_bytes())
//     }
// }

// impl FromStr for PublicKeyBuf {
//     type Err = ParseError;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let bytes_vec = STANDARD
//             .decode(s)
//             .map_err(|_| ParseError::NotBase64(s.to_string()))?;
//         let bytes: &[u8; 32] = bytes_vec
//             .as_slice()
//             .try_into()
//             .map_err(|_| ParseError::IncorrectLength(s.to_string()))?;
//         Ok(Self {
//             key: bytes.clone().into(),
//         })
//     }
// }

// impl Serialize for PublicKeyBuf {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let s: String = self.into();
//         s.serialize(serializer)
//     }
// }

// impl<'de> Deserialize<'de> for PublicKeyBuf {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let string = String::deserialize(deserializer)?;
//         FromStr::from_str(&string).map_err(|e|D::Error::custom(format!("{e}")))
//     }
// }

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
