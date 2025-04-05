use anyhow::Result;
use esp_idf_svc::hal::delay;
use spec::{wifi::WifiActions, CalcRequest};
use state::State;

pub mod spec;
pub mod state;

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
    state.push_incoming(CalcRequest::Wifi(WifiActions::Scan));

    while state.is_processing() {
        state.try_process_incoming();

        let res = state.poll_processing();
        println!("Response: {:?}", res);
        delay::Ets::delay_ms(500);
    }

    Ok(())
}
