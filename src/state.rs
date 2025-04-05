use std::{collections::VecDeque, future::poll_fn, task::Poll};

use anyhow::Result;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::prelude::Peripherals,
    nvs::EspDefaultNvsPartition,
    timer::EspTaskTimerService,
    wifi::{AsyncWifi, EspWifi},
};
use futures::{executor, future::BoxFuture, FutureExt};

use crate::spec::{CalcRequest, CalcResponse};

pub struct State {
    pub wifi: *mut AsyncWifi<EspWifi<'static>>,
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

        Ok(Self {
            // Drop is implemented in and so this is safe :)
            wifi: Box::into_raw(Box::new(AsyncWifi::wrap(wifi, sysloop, timer_service)?)),
            processing: None,
            incoming: VecDeque::new(),
        })
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

    pub fn is_processing(&self) -> bool {
        self.processing.is_some() || !self.incoming.is_empty()
    }
}

impl Drop for State {
    fn drop(&mut self) {
        let _box = unsafe { Box::from_raw(self.wifi) };
    }
}
