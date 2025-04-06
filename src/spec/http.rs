use anyhow::bail;
use esp_idf_svc::{
    http::{client::EspHttpConnection, Method},
    io::{utils::try_read_full, EspIOError},
};

use embedded_svc::http::client::Client;

use super::{Deserialise, Serialise};
use crate::safe_read::SafeRead;

type Headers = Vec<(String, String)>;
type HttpClient = Client<EspHttpConnection>;

pub trait HeadersTrait {
    fn as_full_ref(&self) -> Vec<(&str, &str)>;
}

impl HeadersTrait for Headers {
    fn as_full_ref(&self) -> Vec<(&str, &str)> {
        self.iter()
            .map(|v| (v.0.as_str(), v.1.as_str()))
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone)]
pub enum MethodWithArgs {
    Delete,
    Get,
    Head(Headers),
    Post(Headers),
    Put(Headers),
}

impl MethodWithArgs {
    pub fn request<'a>(
        self,
        client: &'a mut HttpClient,
        uri: &'a str,
    ) -> Result<HttpResp, EspIOError> {
        // Bit confusing but we need to get the lifetimes correct
        let headers = self.headers();

        let request = match self {
            Self::Delete => {
                println!("-> DELETE {uri}");
                client.delete(uri)
            }
            Self::Get => {
                println!("-> GET {uri}");
                client.get(uri)
            }
            Self::Head(_) => {
                println!("-> HEAD {uri}");
                client.request(Method::Head, uri, headers.as_ref().unwrap())
            }
            Self::Post(_) => {
                println!("-> POST {uri}");
                client.post(uri, headers.as_ref().unwrap())
            }
            Self::Put(_) => {
                println!("-> PUT {uri}");
                client.put(uri, headers.as_ref().unwrap())
            }
        }?;

        let mut response = request.submit()?;
        let mut buf = [0u8; 4096];
        let bytes_read = try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;

        Ok(HttpResp {
            raw: Vec::from_iter(buf.into_iter().take(bytes_read)),
        })
    }

    fn headers(&self) -> Option<Vec<(&str, &str)>> {
        match self {
            Self::Delete | Self::Get => None,
            Self::Put(h) | Self::Head(h) | Self::Post(h) => Some(h.as_full_ref()),
        }
    }
}

impl Deserialise for MethodWithArgs {
    fn from_bytes<R: esp_idf_svc::io::Read>(src: &mut R) -> anyhow::Result<Self> {
        Ok(match src.try_next()? {
            0 => Self::Delete,
            1 => Self::Get,
            2 => Self::Head(Headers::from_bytes(src)?),
            3 => Self::Post(Headers::from_bytes(src)?),
            4 => Self::Put(Headers::from_bytes(src)?),
            i => bail!("Unknown id: {i} when trying to decode MethodWithArgs"),
        })
    }
}

#[derive(Debug, Clone)]
pub struct HttpReq {
    url: String,
    extra: MethodWithArgs,
}

impl HttpReq {
    pub fn send(self, client: &mut HttpClient) -> Result<HttpResp, EspIOError> {
        self.extra.request(client, &self.url)
    }
}

impl Deserialise for HttpReq {
    fn from_bytes<R: esp_idf_svc::io::Read>(src: &mut R) -> anyhow::Result<Self> {
        let url = String::from_bytes(src)?;
        let extra = MethodWithArgs::from_bytes(src)?;

        Ok(Self { url, extra })
    }
}

#[derive(Debug, Clone)]
pub struct HttpResp {
    raw: Vec<u8>,
}

impl Serialise for HttpResp {
    fn to_bytes(self) -> Vec<u8> {
        let mut v = Vec::with_capacity(self.raw.len() + 4);

        v.extend((self.raw.len() as u32).to_be_bytes());

        v.extend(self.raw);

        v
    }
}

// TODO:
// - Implement Deserialise for HttpReq
