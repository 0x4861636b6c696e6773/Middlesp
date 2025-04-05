use esp_idf_svc::{
    io::Read,
    wifi::{AuthMethod, ClientConfiguration, PmfConfiguration},
};
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
    Unknown,
}

impl Deserialise for CalcRequest {
    fn from_bytes<R: Read>(src: &mut R) -> Result<Self, R::Error> {
        let mut buf: [u8; 1] = [0];
        let size = src.read(&mut buf)?;
        if size == 0 {
            return Ok(Self::Unknown);
        }

        Ok(match buf[0] {
            0 => Self::Wifi(WifiActions::from_bytes(src)?),
            1 => Self::Wifi(HttpReq::from_bytes(src)?),
            _ => Self::Unknown,
        })
    }
}

impl Deserialise for ClientConfiguration {
    fn from_bytes<R: Read>(src: &mut R) -> Result<Self, R::Error> {
        const SSID_SIZE: usize = std::mem::size_of::<heapless::String<32>>();
        const PASS_SIZE: usize = std::mem::size_of::<heapless::String<64>>();
        const SIZE: usize = SSID_SIZE + PASS_SIZE + 1;
        let mut buf = [0_u8; SIZE];
        let len = src.read(&mut buf)?;
        if len != SIZE {
            println!("Failed to decode client config, returning default");
            return Ok(ClientConfiguration::default());
        }

        let Ok(ssid): Result<heapless::String<32>, _> =
            heapless::String::from_utf8(heapless::Vec::from_slice(&buf[0..SSID_SIZE]).unwrap())
        else {
            println!("Failed to decode ssid, returning default");
            return Ok(ClientConfiguration::default());
        };

        let Ok(pass): Result<heapless::String<64>, _> = heapless::String::from_utf8(
            heapless::Vec::from_slice(&buf[SSID_SIZE..SSID_SIZE + PASS_SIZE]).unwrap(),
        ) else {
            println!("Failed to decode ssid, returning default");
            return Ok(ClientConfiguration::default());
        };

        let auth = match buf[SSID_SIZE + PASS_SIZE] {
            1 => AuthMethod::WEP,
            2 => AuthMethod::WPA,
            3 => AuthMethod::WPA2Personal,
            4 => AuthMethod::WPAWPA2Personal,
            5 => AuthMethod::WPA2Enterprise,
            6 => AuthMethod::WPA3Personal,
            7 => AuthMethod::WPA2WPA3Personal,
            8 => AuthMethod::WAPIPersonal,
            _ => AuthMethod::None,
        };

        Ok(ClientConfiguration {
            ssid,
            bssid: None,
            auth_method: auth,
            password: pass,
            channel: None,
            scan_method: esp_idf_svc::wifi::ScanMethod::FastScan,
            pmf_cfg: PmfConfiguration::new_pmf_optional(),
        })
    }
}

#[derive(Debug)]
pub enum CalcResponse {
    Wifi(WifiResponse),
    Http(HttpResp),
}

impl CalcResponse {
    pub const fn id(&self) -> u8 {
        match self {
            Self::Wifi(_) => 0,
            Self::Http(_) => 1,
        }
    }

    fn serialise_child(&self) -> Vec<u8> {
        match self {
            Self::Wifi(resp) => resp.to_bytes(),
            Self::Http(resp) => resp.to_bytes(),
        }
    }
}

impl Serialise for CalcResponse {
    fn to_bytes(&self) -> Vec<u8> {
        let mut v = vec![self.id()];

        v.extend(self.serialise_child());

        v
    }
}
