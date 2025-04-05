use serde::{Deserialize, Serialize};
use wifi::{WifiActions, WifiResponse};

pub mod wifi;

#[derive(Debug, Serialize, Deserialize)]
pub enum CalcRequest {
    Wifi(WifiActions),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CalcResponse {
    Wifi(WifiResponse),
}
