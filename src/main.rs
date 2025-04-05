use anyhow::Result;
use esp_idf_svc::hal::delay;
use spec::{wifi::WifiActions, CalcRequest};
use state::State;

pub mod spec;
pub mod state;

/// NOTE: It seems we can actually use two threads (and make this a bunch
/// nicer with having one thread on reading incoming and the other just waiting
/// on the wifi requests: https://esp32.implrust.com/wifi/embassy/http-request.html)
///
/// This is the main function :)
fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let mut state = State::new()?;
    state.push_incoming(CalcRequest::Wifi(WifiActions::SetConfig(
        esp_idf_svc::wifi::ClientConfiguration::default(),
    )));
    state.push_incoming(CalcRequest::Wifi(WifiActions::Start));

    while state.is_processing() {
        state.read_incoming();

        state.try_process_incoming();

        state.try_send_processing();
        // let res = state.poll_processing();
        // println!("Response: {:?}", res);
        delay::Ets::delay_ms(100);
    }

    Ok(())
}
