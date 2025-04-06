use crate::safe_read::SafeRead;
use anyhow::bail;
use esp_idf_svc::io::{EspIOError, Read};
use http::{HttpReq, HttpResp};
use wifi::{WifiActions, WifiResponse};

mod http;
mod serialise;
pub mod wifi;

pub use serialise::{Deserialise, Serialise};

#[derive(Debug, Clone)]
pub enum CalcRequest {
    Wifi(WifiActions),
    Http(HttpReq),
}

impl Deserialise for CalcRequest {
    fn from_bytes<R: Read>(src: &mut R) -> anyhow::Result<Self> {
        let id = src.try_read::<1>()?[0];
        Ok(match id {
            0 => Self::Wifi(WifiActions::from_bytes(src)?),
            1 => Self::Http(HttpReq::from_bytes(src)?),
            _ => bail!("Could not match {id} to CalcRequest"),
        })
    }
}

#[derive(Debug)]
pub enum CalcResponse {
    Wifi(WifiResponse),
    Http(Result<HttpResp, EspIOError>),
}

impl CalcResponse {
    pub const fn id(&self) -> u8 {
        match self {
            Self::Wifi(_) => 0,
            Self::Http(_) => 1,
        }
    }

    fn serialise_child(self) -> Vec<u8> {
        match self {
            Self::Wifi(resp) => resp.to_bytes(),
            Self::Http(resp) => resp.to_bytes(),
        }
    }
}

impl Serialise for CalcResponse {
    fn to_bytes(self) -> Vec<u8> {
        let mut v = vec![self.id()];

        v.extend(self.serialise_child());

        v
    }
}
