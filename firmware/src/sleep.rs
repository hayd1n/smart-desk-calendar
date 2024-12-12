pub mod common;

#[allow(unreachable_code)]
fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Entering deep sleep");
    unsafe {
        esp_idf_svc::sys::esp_deep_sleep_start();
    }
    log::error!("Deep sleep failed");
}
