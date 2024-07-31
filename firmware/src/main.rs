#![feature(duration_constructors)]
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Timelike, Utc};
use chrono_tz::{Asia::Taipei, Tz};
use embedded_graphics::{draw_target::DrawTarget, prelude::Point};
use epd_waveshare::{
    color::Color::{Black as White, White as Black},
    epd7in5_v2::{Epd7in5, HEIGHT, WIDTH},
    graphics::VarDisplay,
    prelude::*,
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        delay::{self, Delay},
        gpio::{self, PinDriver},
        prelude::*,
        reset::{self, WakeupReason},
        spi,
    },
    nvs::EspDefaultNvsPartition,
    sntp::{EspSntp, SyncStatus},
    wifi::{
        AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi, PmfConfiguration,
        ScanMethod, ScanSortMethod,
    },
};
use u8g2_fonts::{
    types::{FontColor, HorizontalAlignment, VerticalPosition},
    FontRenderer,
};

mod font;

pub fn draw_text(display: &mut VarDisplay<Color>, text: &str, font: FontRenderer, x: i32, y: i32) {
    font.render_aligned(
        text,
        Point::new(x, y),
        VerticalPosition::Top,
        HorizontalAlignment::Left,
        FontColor::Transparent(Black),
        display,
    )
    .unwrap();
}

fn enter_deep_sleep(sleep_time: Duration) {
    log::info!("entering deep sleep");
    unsafe {
        esp_idf_sys::esp_deep_sleep(sleep_time.as_micros() as u64);
    }
}

fn get_time() -> DateTime<Tz> {
    // Obtain System Time
    let st_now = SystemTime::now();
    // Convert to UTC Time
    let dt_now_utc: DateTime<Utc> = st_now.clone().into();
    dt_now_utc.with_timezone(&Taipei)
}

fn main() -> anyhow::Result<()> {
    const WIFI_SSID: &'static str = env!("WIFI_SSID");
    const WIFI_PASSWORD: &'static str = env!("WIFI_PASSWORD");

    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let wakeup_reason = reset::WakeupReason::get();

    log::info!("wakeup: {:?}", wakeup_reason);

    log::info!("ssid: {}, password: {}", WIFI_SSID, WIFI_PASSWORD);

    let mut delay = delay::Ets;

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let pins = peripherals.pins;

    let spi_p = peripherals.spi3;
    let sclk = pins.gpio13;
    let sdo = pins.gpio14;

    let busy_in = pins.gpio25;
    let rst = pins.gpio26;
    let dc = pins.gpio27;

    let mut pwr = PinDriver::output(pins.gpio23)?;
    pwr.set_high()?;

    let mut driver = spi::SpiDeviceDriver::new_single(
        spi_p,
        sclk,
        sdo,
        Option::<gpio::AnyIOPin>::None,
        Option::<gpio::AnyOutputPin>::None,
        &spi::config::DriverConfig::new(),
        &spi::config::Config::new().baudrate(10.MHz().into()),
    )?;

    log::info!("driver setup completed");

    // Setup EPD
    let mut epd_driver = Epd7in5::new(
        &mut driver,
        PinDriver::input(busy_in)?,
        PinDriver::output(dc)?,
        PinDriver::output(rst)?,
        &mut delay,
        None,
    )
    .unwrap();

    log::info!("epd setup completed");

    const BUFFER_SIZE: usize = epd_waveshare::buffer_len(WIDTH as usize, HEIGHT as usize);

    let mut buffer = vec![0u8; BUFFER_SIZE];

    let mut display = VarDisplay::<Color>::new(WIDTH, HEIGHT, &mut buffer, false)
        .expect("failed to create display");

    // let mut display = Display7in5::default();
    // let mut display = Display5in83::default();

    let font_inter_bold_48 = FontRenderer::new::<font::inter_bold_48_48>();
    let font_inter_bold_64 = FontRenderer::new::<font::inter_bold_64_64>();

    if wakeup_reason != WakeupReason::Timer {
        let mut wifi = BlockingWifi::wrap(
            EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs))?,
            sysloop,
        )?;

        wifi.set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: WIFI_SSID.try_into().unwrap(),
            bssid: None,
            auth_method: AuthMethod::WPA2Personal,
            password: WIFI_PASSWORD.try_into().unwrap(),
            channel: None,
            scan_method: ScanMethod::CompleteScan(ScanSortMethod::Signal),
            pmf_cfg: PmfConfiguration::default(),
        }))?;

        // Start Wifi
        wifi.start()?;

        // Connect Wifi
        wifi.connect()?;

        // Wait until the network interface is up
        wifi.wait_netif_up()?;

        // Print Out Wifi Connection Configuration
        while !wifi.is_connected().unwrap() {
            // Get and print connection configuration
            let config = wifi.get_configuration().unwrap();
            println!("Waiting for station {:?}", config);
        }

        log::info!("Wifi Connected");

        // Create Handle and Configure SNTP
        let ntp = EspSntp::new_default().unwrap();

        // Synchronize NTP
        log::info!("Synchronizing with NTP Server");
        while ntp.get_sync_status() != SyncStatus::Completed {}
        log::info!("Time Sync Completed");

        wifi.disconnect()?;
        wifi.stop()?;
    }

    let now = get_time();

    let time = format!("{}", now.format("%H:%M"));
    let meridiem = format!("{}", now.format("%p"));

    display.clear(White)?;
    log::info!("Display clear complete");

    draw_text(&mut display, time.as_str(), font_inter_bold_64, 35, 28);
    draw_text(&mut display, meridiem.as_str(), font_inter_bold_48, 217, 40);
    log::info!("Draw text complete");

    epd_driver
        .update_and_display_frame(&mut driver, display.buffer(), &mut delay)
        .expect("display frame");
    log::info!("Update and display frame complete");

    Delay::new_default().delay_ms(5000);
    pwr.set_low()?;
    log::info!("All complete");

    let now = get_time();

    enter_deep_sleep(Duration::from_mins(5) - Duration::from_secs(now.second() as u64));

    loop {
        log::info!(".");
        Delay::new_default().delay_ms(5000);
    }
}
