use esp_idf_svc::{
    io::{EspIOError, Read},
    wifi::{AccessPointInfo, AuthMethod, ClientConfiguration, PmfConfiguration},
};

use crate::safe_read::SafeRead;

pub trait Serialise {
    fn to_bytes(self) -> Vec<u8>;
}

pub trait Deserialise: Sized {
    fn from_bytes<R: Read>(src: &mut R) -> anyhow::Result<Self>;
}

impl<T: Serialise + Sized> Serialise for Vec<T> {
    fn to_bytes(self) -> Vec<u8> {
        let mut v = vec![self.len() as u8];
        v.extend(self.into_iter().flat_map(Serialise::to_bytes));

        v
    }
}

impl Serialise for AccessPointInfo {
    fn to_bytes(self) -> Vec<u8> {
        let mut v = Vec::with_capacity(std::mem::size_of::<Self>());

        v.extend(self.ssid.as_bytes());
        v.extend(self.bssid);
        v.push(self.channel);
        v.extend(self.signal_strength.to_be_bytes());

        v
    }
}

impl<T: Serialise> Serialise for Result<T, EspIOError> {
    fn to_bytes(self) -> Vec<u8> {
        match self {
            Ok(t) => {
                let res = t.to_bytes();
                let mut vec = Vec::with_capacity(res.len() + 1);
                vec.push(0);
                vec.extend(res);
                vec
            }
            Err(e) => {
                println!("Throughing away error when sending: {e:?}");
                vec![1] // Simply throw away the errors
            }
        }
    }
}

impl Deserialise for ClientConfiguration {
    fn from_bytes<R: Read>(src: &mut R) -> anyhow::Result<Self> {
        const SSID_SIZE: usize = std::mem::size_of::<heapless::String<32>>();
        const PASS_SIZE: usize = std::mem::size_of::<heapless::String<64>>();

        let Ok(ssid): Result<heapless::String<32>, _> = heapless::String::from_utf8(
            heapless::Vec::from_slice(&src.try_read::<SSID_SIZE>()?).unwrap(),
        ) else {
            println!("Failed to decode ssid, returning default");
            return Ok(ClientConfiguration::default());
        };

        let Ok(pass): Result<heapless::String<64>, _> = heapless::String::from_utf8(
            heapless::Vec::from_slice(&src.try_read::<PASS_SIZE>()?).unwrap(),
        ) else {
            println!("Failed to decode ssid, returning default");
            return Ok(ClientConfiguration::default());
        };

        let auth = match src.try_read::<1>()?[0] {
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

impl<A: Deserialise, B: Deserialise> Deserialise for (A, B) {
    fn from_bytes<R: Read>(src: &mut R) -> anyhow::Result<Self> {
        Ok((A::from_bytes(src)?, B::from_bytes(src)?))
    }
}

impl<T: Deserialise> Deserialise for Vec<T> {
    fn from_bytes<R: Read>(src: &mut R) -> anyhow::Result<Self> {
        // Get the length of the vector
        let len = src.try_next()?;
        let mut res = Vec::with_capacity(len as usize);

        for _ in 0..len {
            res.push(T::from_bytes(src)?)
        }

        Ok(res)
    }
}

impl Deserialise for String {
    fn from_bytes<R: Read>(src: &mut R) -> anyhow::Result<Self> {
        // Get the length of the string
        let len = u32::from_be_bytes(src.try_read::<4>()?);

        Ok(String::from_utf8(src.try_read_dyn(len as usize)?)?)
    }
}
