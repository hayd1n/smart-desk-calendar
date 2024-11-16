#![allow(dead_code)]

use std::time::Duration;

use chrono_tz::Tz;
use embedded_graphics::{
    mono_font::{ascii::FONT_9X18, MonoTextStyle},
    prelude::Point,
    text::Text,
    Drawable,
};
use epd_waveshare::{color::Color, graphics::VarDisplay, prelude::WaveshareDisplay};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::reset,
    nvs::EspDefaultNvsPartition,
    sntp::{EspSntp, SyncStatus},
    wifi,
};
use gui::page::main_page::MainPage;

use crate::{
    board::Board,
    common::get_time,
    display::{create_display, Black},
};

#[derive(Debug)]
pub enum Mode {
    Initialize,
    Setting,
    Normal,
    LowPower,
}

#[derive(Debug, Clone)]
pub struct AppConfig {}

#[derive(Debug, Clone)]
pub struct AppSettings {
    // Wi-Fi
    pub wifi_ssid: heapless::String<32>,
    pub wifi_password: heapless::String<64>,
    pub wifi_auth_method: wifi::AuthMethod,
    // Timezone
    pub timezone: Tz,
    // Misc
    pub refresh_interval: Duration,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            wifi_ssid: env!("WIFI_SSID").try_into().unwrap(),
            wifi_password: env!("WIFI_PASSWORD").try_into().unwrap(),
            wifi_auth_method: env!("WIFI_AUTH_MODE").try_into().unwrap(),
            timezone: chrono_tz::Asia::Taipei,
            refresh_interval: Duration::from_mins(5),
        }
    }
}

pub struct App {
    mode: Mode,
    config: AppConfig,
    settings: AppSettings,

    board: Board,
    sysloop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,

    display: VarDisplay<'static, Color>,
}

impl App {
    pub fn new(board: Board, sysloop: EspSystemEventLoop, nvs: EspDefaultNvsPartition) -> Self {
        Self {
            mode: Mode::Normal,
            config: AppConfig {},
            settings: AppSettings::default(),
            board,
            sysloop,
            nvs,
            display: create_display().unwrap(),
        }
    }

    fn initialize(&mut self) -> anyhow::Result<()> {
        log::info!("initialize");

        // Turn on the LED
        self.board.led.set_high()?;

        let wakeup_reason = reset::WakeupReason::get();
        match wakeup_reason {
            reset::WakeupReason::Unknown | reset::WakeupReason::Timer => self.mode = Mode::Normal,
            _ => log::info!("wakeup: {:?}", wakeup_reason),
        }

        Ok(())
    }

    #[allow(unreachable_code)]
    fn sleep(&mut self, sleep_time: Duration) -> anyhow::Result<()> {
        // Turn off the LED
        self.board.led.set_low()?;

        log::info!("Entering deep sleep");
        log::info!("It will wake up in {} seconds.", sleep_time.as_secs());
        unsafe {
            esp_idf_svc::sys::esp_deep_sleep(sleep_time.as_micros() as u64);
        }
        log::error!("Deep sleep failed");

        Ok(())
    }

    fn update_and_display(&mut self) -> anyhow::Result<()> {
        self.board.epd.update_and_display_frame(
            &mut self.board.spi,
            self.display.buffer(),
            &mut self.board.delay,
        )?;
        self.board.delay.delay_ms(5000);
        log::info!("Update and display");
        Ok(())
    }

    // Show error message on the screen
    fn handle_error(&mut self, e: anyhow::Error) {
        log::error!("Unexpected error: {:?}", e);

        // Create a new character style
        let style = MonoTextStyle::new(&FONT_9X18, Black);

        // Create a text at position (20, 30) and draw it using the previously defined style
        Text::new(
            &format!("Unexpected error: {:?}", e),
            Point::new(20, 20),
            style,
        )
        .draw(&mut self.display)
        .unwrap();

        // Update and display the frame
        self.update_and_display().unwrap();
    }

    fn run_internal(&mut self) -> anyhow::Result<()> {
        self.initialize()?;

        match self.mode {
            Mode::Normal => {
                self.normal_initialize()?;
                self.normal_update()?;
            }
            _ => unimplemented!(),
        };

        self.sleep(self.settings.refresh_interval)?;

        Ok(())
    }

    pub fn run(&mut self) {
        if let Err(e) = self.run_internal() {
            self.handle_error(e);
        }
    }
}

impl App {
    fn normal_initialize_wifi(&mut self) -> anyhow::Result<()> {
        self.board
            .wifi
            .set_configuration(&wifi::Configuration::Client(wifi::ClientConfiguration {
                ssid: self.settings.wifi_ssid.clone(),
                bssid: None,
                auth_method: self.settings.wifi_auth_method,
                password: self.settings.wifi_password.clone(),
                channel: None,
                scan_method: wifi::ScanMethod::FastScan,
                pmf_cfg: wifi::PmfConfiguration::default(),
            }))?;

        // Start Wifi
        self.board.wifi.start()?;

        // Connect Wifi
        self.board.wifi.connect()?;

        // Wait until the network interface is up
        self.board.wifi.wait_netif_up()?;

        Ok(())
    }

    fn normal_sync_ntp(&mut self) -> anyhow::Result<()> {
        // Create Handle and Configure SNTP
        let ntp = EspSntp::new_default().unwrap();
        log::info!("Synchronizing with NTP Server");
        // Wait until the time is synchronized
        while ntp.get_sync_status() != SyncStatus::Completed {}
        log::info!("Time Sync Completed");
        Ok(())
    }

    fn normal_initialize(&mut self) -> anyhow::Result<()> {
        log::info!("NormalMode::initialize");
        self.normal_initialize_wifi()?;
        self.normal_sync_ntp()?;
        Ok(())
    }

    fn normal_update(&mut self) -> anyhow::Result<()> {
        log::info!("NormalMode::update");

        // Get the current time
        let now_local = get_time()
            .with_timezone(&self.settings.timezone)
            .naive_local();
        let weekday = format!("{}", now_local.format("%A"));

        // Create a new main page
        let mut main_page = MainPage::new(now_local);

        // Set weekday
        main_page.set_weekday(weekday);

        // Render the main page
        main_page.draw(&mut self.display)?;

        // Update and display the frame
        self.update_and_display()?;

        Ok(())
    }
}
