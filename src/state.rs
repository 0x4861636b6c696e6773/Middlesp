use std::{collections::VecDeque, future::poll_fn, task::Poll};

use anyhow::Result;
use embassy_net::{
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        gpio,
        prelude::Peripherals,
        uart::{config, UartDriver},
        units::Hertz,
    },
    nvs::EspDefaultNvsPartition,
    timer::EspTaskTimerService,
    wifi::{AsyncWifi, EspWifi},
};
use futures::{executor, future::BoxFuture, FutureExt};
use reqwless::client::{HttpClient, TlsConfig};

use crate::spec::{CalcRequest, CalcResponse, Deserialise, Serialise};

pub struct State {
    wifi: *mut AsyncWifi<EspWifi<'static>>,
    uart: *mut UartDriver<'static>,
    http: *mut HttpClient<'static>,
    processing: Option<BoxFuture<'static, CalcResponse>>,
    incoming: VecDeque<CalcRequest>,
}

impl State {
    pub fn new() -> Result<Self> {
        let peripherals = Peripherals::take().unwrap();
        let sysloop = EspSystemEventLoop::take()?;
        let nvs = EspDefaultNvsPartition::take()?;

        let wifi = EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs))?;
        let timer_service = EspTaskTimerService::new()?;

        // Create uart (Serial interaction)
        let tx = peripherals.pins.gpio5;
        let rx = peripherals.pins.gpio6;

        let config = config::Config::new().baudrate(Hertz(115_200));
        let uart = UartDriver::new(
            peripherals.uart1,
            tx,
            rx,
            Option::<gpio::Gpio0>::None,
            Option::<gpio::Gpio1>::None,
            &config,
        )
        .unwrap();

        // Create http client
        // TODO: following: https://esp32.implrust.com/wifi/embassy/http-request.html
        // But i don't think including the esp-hal package (which has Rng is a good idea)
        let tls_seed = rng.random() as u64 | ((rng.random() as u64) << 32);
        let mut rx_buffer = [0; 4096];
        let mut tx_buffer = [0; 4096];
        let dns = DnsSocket::new(&stack);
        let tcp_state = TcpClientState::<1, 4096, 4096>::new();
        let tcp = TcpClient::new(stack, &tcp_state);

        let tls = TlsConfig::new(
            tls_seed,
            &mut rx_buffer,
            &mut tx_buffer,
            reqwless::client::TlsVerify::None,
        );

        let mut client = HttpClient::new_with_tls(&tcp, &dns, tls);

        Ok(Self {
            uart: Box::into_raw(Box::new(uart)),
            // Drop is implemented in and so this is safe :)
            wifi: Box::into_raw(Box::new(AsyncWifi::wrap(wifi, sysloop, timer_service)?)),
            http: Box::into_raw(Box::new(client)),
            processing: None,
            incoming: VecDeque::new(),
        })
    }

    pub fn uart(&mut self) -> &mut UartDriver<'static> {
        unsafe { self.uart.as_mut().unwrap() }
    }

    pub fn wifi(&mut self) -> &mut AsyncWifi<EspWifi<'static>> {
        unsafe { self.wifi.as_mut().unwrap() }
    }

    pub fn http(&mut self) -> &mut HttpClient<'static> {
        unsafe { self.http.as_mut().unwrap() }
    }

    pub fn read_incoming(&mut self) {
        match CalcRequest::from_bytes(self.uart()) {
            Ok(res) => {
                println!("Receieved: {res:?}");
                self.push_incoming(res);
            }
            Err(e) => println!("Failed to get buf: {e:?}"),
        }
    }

    pub fn push_incoming(&mut self, req: CalcRequest) {
        self.incoming.push_back(req);
    }

    /// Processes the next incoming request
    pub fn try_process_incoming(&mut self) {
        if self.processing.is_some() {
            return;
        }

        if let Some(next) = self.incoming.pop_front() {
            let wifi = self.wifi;
            let resp = match next {
                CalcRequest::Wifi(action) => action
                    .run_on(unsafe { wifi.as_mut().unwrap() })
                    .map(CalcResponse::Wifi)
                    .boxed(),
                CalcRequest::Unknown => panic!("Unknown state"),
            };

            self.processing = Some(resp);
        }
    }

    pub fn poll_processing(&mut self) -> Option<CalcResponse> {
        if let Some(processing) = &mut self.processing {
            let res = executor::block_on(poll_fn(|ctx| {
                Poll::Ready(match processing.poll_unpin(ctx) {
                    Poll::Ready(v) => Some(v),
                    Poll::Pending => None,
                })
            }));

            if res.is_some() {
                self.processing = None;
            }

            res
        } else {
            None
        }
    }

    pub fn try_send_processing(&mut self) {
        if let Some(resp) = self.poll_processing() {
            println!("Sending: {resp:?}");

            let buf = resp.to_bytes();
            match self.uart().write(&buf) {
                Ok(size) if size != buf.len() => {
                    println!("Only write {size} bytes when expected {}", buf.len());
                }
                Err(e) => {
                    println!("Error when writing to stream: {e:?}")
                }
                _ => {} // Everything is fine with the world
            }
        }
    }

    pub fn is_processing(&self) -> bool {
        self.processing.is_some() || !self.incoming.is_empty()
    }
}

impl Drop for State {
    fn drop(&mut self) {
        let _box = unsafe { Box::from_raw(self.wifi) };
        let _box = unsafe { Box::from_raw(self.uart) };
        let _box = unsafe { Box::from_raw(self.http) };
    }
}
