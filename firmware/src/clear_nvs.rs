use common::NVS_NAMESPACE;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};

pub mod common;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut nvs_storage = match EspNvs::new(nvs, NVS_NAMESPACE, true) {
        Ok(nvs) => nvs,
        Err(e) => panic!("Could't get namespace {:?}", e),
    };

    nvs_storage
        .remove("settings")
        .expect("Could not remove settings");

    log::info!("NVS cleared");
}
