use std::future::{self, Future};

use embedded_svc::wifi::{self, ClientConfiguration};
use enumset::EnumSet;
use esp_idf_svc::{
    sys::EspError,
    wifi::{AccessPointInfo, AsyncWifi, Capability, EspWifi},
};
use futures::{future::BoxFuture, FutureExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WifiActions {
    /// [esp_idf_svc::wifi::AsyncWifi::is_started]
    IsStarted,
    /// [esp_idf_svc::wifi::AsyncWifi::is_connected]
    IsConnected,
    /// [esp_idf_svc::wifi::AsyncWifi::get_capabilities]
    GetCapabilities,
    /// [esp_idf_svc::wifi::AsyncWifi::start]
    Start,
    /// [esp_idf_svc::wifi::AsyncWifi::stop]
    Stop,
    /// [esp_idf_svc::wifi::AsyncWifi::scan]
    Scan,
    /// [esp_idf_svc::wifi::AsyncWifi::connect]
    Connect,
    /// [esp_idf_svc::wifi::AsyncWifi::disconnect]
    Disconnect,
    /// [esp_idf_svc::wifi::AsyncWifi::set_configuration]
    SetConfig(ClientConfiguration),
}

impl WifiActions {
    pub fn run_on<'a>(self, wifi: &'a mut AsyncWifi<EspWifi<'_>>) -> BoxFuture<'a, WifiResponse> {
        match self {
            Self::IsStarted => {
                future::ready(wifi.is_started().into_resp(WifiResponse::IsStarted)).boxed()
            }
            Self::Scan => wifi.scan().into_resp(WifiResponse::AccessPoints).boxed(),
            Self::IsConnected => {
                future::ready(wifi.is_connected().into_resp(WifiResponse::IsConnected)).boxed()
            }
            Self::GetCapabilities => {
                future::ready(wifi.get_capabilities().into_resp_or(WifiResponse::Started)).boxed()
            }
            Self::Start => wifi.start().into_resp_or(WifiResponse::Started).boxed(),
            Self::Stop => wifi.stop().into_resp_or(WifiResponse::Stopped).boxed(),
            Self::Connect => wifi.connect().into_resp_or(WifiResponse::Connected).boxed(),
            Self::Disconnect => wifi
                .disconnect()
                .into_resp_or(WifiResponse::Disconnected)
                .boxed(),
            Self::SetConfig(config) => future::ready(
                wifi.set_configuration(&wifi::Configuration::Client(config))
                    .into_resp_or(WifiResponse::Configured),
            )
            .boxed(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HttpActions {
    Send,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WifiResponse {
    Error(i32),
    IsStarted(bool),
    IsConnected(bool),
    AccessPoints(Vec<AccessPointInfo>),
    Capabilities(EnumSet<Capability>),
    Started,
    Stopped,
    Connected,
    Disconnected,
    Configured,
}

impl WifiResponse {
    #[inline]
    pub fn new_error(err: EspError) -> Self {
        Self::Error(err.code())
    }
}

pub trait ConvertToWifiResponse<T> {
    fn into_resp(self, f: impl Fn(T) -> WifiResponse) -> WifiResponse;
    fn into_resp_or(self, or: WifiResponse) -> WifiResponse;
}

impl<T> ConvertToWifiResponse<T> for Result<T, EspError> {
    #[inline]
    fn into_resp(self, f: impl Fn(T) -> WifiResponse) -> WifiResponse {
        match self {
            Ok(r) => f(r),
            Err(e) => WifiResponse::new_error(e),
        }
    }

    #[inline]
    fn into_resp_or(self, or: WifiResponse) -> WifiResponse {
        match self {
            Ok(_) => or,
            Err(e) => WifiResponse::new_error(e),
        }
    }
}

pub trait AsyncConvertToWifiResponse<T> {
    fn into_resp(self, f: impl Fn(T) -> WifiResponse) -> impl Future<Output = WifiResponse>;
    fn into_resp_or(self, or: WifiResponse) -> impl Future<Output = WifiResponse>;
}

impl<T, F: Future<Output = Result<T, EspError>>> AsyncConvertToWifiResponse<T> for F {
    #[inline]
    fn into_resp(self, f: impl Fn(T) -> WifiResponse) -> impl Future<Output = WifiResponse> {
        self.map(|r| r.into_resp(f))
    }

    #[inline]
    fn into_resp_or(self, or: WifiResponse) -> impl Future<Output = WifiResponse> {
        self.map(|r| r.into_resp_or(or))
    }
}
