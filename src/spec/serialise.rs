use esp_idf_svc::{io::Read, wifi::AccessPointInfo};

pub trait Serialise {
    fn to_bytes(&self) -> Vec<u8>;
}

pub trait Deserialise: Sized {
    fn from_bytes<R: Read>(src: &mut R) -> Result<Self, R::Error>;
}

impl<T: Serialise + Sized> Serialise for Vec<T> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut v = vec![self.len() as u8];
        v.extend(self.iter().map(Serialise::to_bytes).flatten());

        v
    }
}

impl Serialise for AccessPointInfo {
    fn to_bytes(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(std::mem::size_of::<Self>());

        v.extend(self.ssid.as_bytes());
        v.extend(self.bssid);
        v.push(self.channel);
        v.extend(self.signal_strength.to_be_bytes());

        v
    }
}
