#![feature(duration_constructors)]

use app::App;
use board::Board;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop, hal::prelude::Peripherals, nvs::EspDefaultNvsPartition,
};

pub mod app;
pub mod board;
pub mod calendar;
pub mod common;
pub mod display;
pub mod http;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let board = Board::init(peripherals, sysloop.clone(), nvs.clone());

    let mut app = App::new(board, sysloop, nvs);
    app.run();
}
