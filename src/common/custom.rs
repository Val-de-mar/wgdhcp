use std::str::FromStr;

use derive_more::{Display, From, FromStr};
use regex::Regex;
use serde::{de, ser, Deserialize, Serialize};
use url::{Host, ParseError};

#[derive(Clone, Debug, Display, From, FromStr, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key(pub String);

const VALID_KEY: &str = "[+/0-9=A-Za-z]+=";

pub fn check_key(key: &str) -> bool {
    let regex = Regex::new(&format!("^{}$", VALID_KEY)).unwrap();
    regex.is_match(key)
}

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use ser::Error;
        if !check_key(&self.0) {
            return Err(S::Error::custom(format!("invalid key: {}", self.0)));
        }
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let res = String::deserialize(deserializer)?;
        use de::Error;
        if !check_key(&res) {
            return Err(D::Error::custom(format!("invalid key: {}", res)));
        }
        Ok(Self(res))
    }
}

#[derive(Clone, Debug)]
pub struct Endpoint {
    pub host: Host,
    pub port: Option<u16>,
}

impl FromStr for Endpoint {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // let gen_error = || D::Error::custom(format!("cannot parse endpoint '{}'", s));
        let (host, port) = match s.split_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (s, None),
        };
        let host = url::Host::parse(host)?;
        let port: Option<u16> = port.map(|x| x.parse()).transpose().map_err(|_| ParseError::InvalidPort)?;

        Ok(Endpoint {
            host: host,
            port: port,
        })
    }
}

impl Serialize for Endpoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {   
        serializer.serialize_str(&
            match self.port {
                Some(port) => format!("{}:{}", self.host, port),
                None => format!("{}", self.host)
            }
        ) 
    }
}

impl<'de> Deserialize<'de> for Endpoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let string = String::deserialize(deserializer)?;
        let gen_error = |_| D::Error::custom(format!("cannot parse endpoint '{}'", &string));
        
        Self::from_str(&string).map_err(gen_error)
    }
}
