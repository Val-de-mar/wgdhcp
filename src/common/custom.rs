use std::str::FromStr;

use serde::{Deserialize, Serialize};
use url::{Host, ParseError};

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
        let port: Option<u16> = port
            .map(|x| x.parse())
            .transpose()
            .map_err(|_| ParseError::InvalidPort)?;

        Ok(Endpoint {
            host: host,
            port: port,
        })
    }
}

impl From<&Endpoint> for String {
    fn from(value: &Endpoint) -> Self {
        match value.port {
            Some(port) => format!("{}:{}", value.host, port),
            None => format!("{}", value.host),
        }
    }
}

impl Serialize for Endpoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let string: String = self.into();
        serializer.serialize_str(&string)
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
