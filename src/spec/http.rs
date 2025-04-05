//! This is following <https://esp32.implrust.com/wifi/embassy/http-request.html>

use reqwless::{client::HttpClient, request::Method};

use super::{Deserialise, Serialise};
pub struct HttpReq {
    method: Method,
    url: String,
}

impl HttpReq {
    pub async fn send(self, client: &mut HttpClient<'static>) -> HttpResp {
        let mut buffer = [0u8; 4096];
        // TODO: deal with unwrap properly
        let mut http_req = client.request(self.method, &self.url).await.unwrap();

        let response = http_req.send(&mut buffer).await.unwrap();
        let res = response.body().read_to_end().await.unwrap();

        HttpResp {
            raw: Vec::from_iter(res),
        }
    }
}

pub struct HttpResp {
    raw: Vec<u8>,
}

impl Serialise for HttpResp {
    fn to_bytes(self) -> Vec<u8> {
        let v = Vec::with_capacity(self.raw.len() + 4);

        v.push((self.raw.len() as u32).to_be_bytes());

        v.extend(self.raw);

        v
    }
}

// TODO:
// - Implement Deserialise for HttpReq
