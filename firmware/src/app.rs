#![allow(dead_code)]

use std::{
    fmt::Debug,
    str::from_utf8,
    sync::mpsc::{self, RecvTimeoutError},
    time::Duration,
};

use chrono_tz::Tz;
use const_random::const_random;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    prelude::Point,
    text::Text,
    Drawable,
};
use epd_waveshare::{color::Color, graphics::VarDisplay, prelude::WaveshareDisplay};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::reset,
    http::{
        server::{Configuration as HttpServerConfig, EspHttpServer},
        Method,
    },
    io::{EspIOError, Write},
    nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault},
    sntp::{EspSntp, SyncStatus},
    wifi,
};
use gui::page::main_page::MainPage;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WifiConfig {
    pub ssid: heapless::String<32>,
    pub password: heapless::String<64>,
    pub auth_method: wifi::AuthMethod,
}

impl WifiConfig {
    pub fn to_client_config(&self) -> wifi::ClientConfiguration {
        wifi::ClientConfiguration {
            ssid: self.ssid.clone(),
            password: self.password.clone(),
            auth_method: self.auth_method,
            scan_method: wifi::ScanMethod::FastScan,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub device_id: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    // Wi-Fi
    pub wifi_config: WifiConfig,
    // Timezone
    pub timezone: Tz,
    // Misc
    pub refresh_interval: Duration,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            wifi_config: WifiConfig {
                ssid: env!("WIFI_SSID").try_into().unwrap(),
                password: env!("WIFI_PASSWORD").try_into().unwrap(),
                auth_method: env!("WIFI_AUTH_MODE").try_into().unwrap(),
            },
            timezone: chrono_tz::Asia::Taipei,
            refresh_interval: Duration::from_mins(5),
        }
    }
}

pub trait AppMode {
    fn new(app: App) -> Self;
    fn run(&mut self);
    fn retrieve(self) -> App;
}

pub struct InitializeMode {
    app: App,
    wifi_ap_config: wifi::AccessPointConfiguration,
    wifi_client_config: wifi::ClientConfiguration,
}

impl InitializeMode {
    fn setup_wifi(&mut self) -> anyhow::Result<()> {
        self.app
            .board
            .wifi
            .set_configuration(&wifi::Configuration::Mixed(
                self.wifi_client_config.clone(),
                self.wifi_ap_config.clone(),
            ))?;

        Ok(())
    }

    fn connect_wifi(&mut self) -> anyhow::Result<()> {
        // Connect Wifi
        self.app.board.wifi.connect()?;
        // Wait until the network interface is up
        self.app.board.wifi.wait_netif_up().ok();
        Ok(())
    }

    fn initialize_wifi(&mut self) -> anyhow::Result<()> {
        // Setup Wifi
        self.setup_wifi()?;
        // Start Wifi
        self.app.board.wifi.start()?;
        Ok(())
    }

    fn initialize(&mut self) -> anyhow::Result<()> {
        log::info!("InitializeMode::initialize");
        self.initialize_wifi()?;

        Ok(())
    }

    fn run_internal(&mut self) -> anyhow::Result<()> {
        // Initialize the app mode
        self.initialize()?;

        // Display the initializing screen
        let display = &mut self.app.display;
        let text_style = MonoTextStyle::new(&FONT_10X20, Black);
        Text::new("Initialize Mode", Point::new(30, 30), text_style).draw(display)?;
        Text::new(
            &format!(
                "WiFi SSID: {}, Password: {}",
                self.wifi_ap_config.ssid, self.wifi_ap_config.password
            ),
            Point::new(30, 50),
            text_style,
        )
        .draw(display)?;

        // Update and display the frame
        self.app.update_and_display()?;

        // TEST: Test HTTP Server
        // HTTP Configuration
        // Create HTTP Server Connection Handle
        let mut httpserver = EspHttpServer::new(&HttpServerConfig::default())?;

        // Define Server Request Handler Behaviour on Get for Root URL
        httpserver.fn_handler(
            "/",
            Method::Get,
            |request| -> core::result::Result<(), EspIOError> {
                let html = "Hello world!";
                let mut response = request.into_ok_response()?;
                response.write_all(html.as_bytes())?;
                Ok(())
            },
        )?;

        let (tx, rx) = mpsc::channel();

        httpserver.fn_handler(
            "/wifi",
            Method::Post,
            move |mut request| -> anyhow::Result<()> {
                let mut buffer = [0_u8; 256];
                let bytes_read = request.read(&mut buffer)?;
                let recv_str = from_utf8(&buffer[0..bytes_read])?;
                let wifi_config_json: WifiConfig = serde_json::from_str(recv_str)?;

                // Send the wifi configuration to the main thread
                tx.send(wifi_config_json).unwrap();

                let html = format!("Received: {}", recv_str);
                let mut response = request.into_ok_response()?;
                response.write_all(html.as_bytes())?;
                Ok(())
            },
        )?;

        log::info!("Initialize mode started");

        loop {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(wifi_config_json) => {
                    log::info!("Received wifi config: {:?}", wifi_config_json);
                    self.wifi_client_config = wifi_config_json.to_client_config();

                    // Re-setup the wifi
                    self.setup_wifi()?;

                    // Connect to the wifi
                    self.connect_wifi()?;

                    if self.app.board.wifi.is_connected()? {
                        log::info!("Connected to Wi-Fi");

                        // Update settings
                        self.app.settings = Some(AppSettings {
                            wifi_config: wifi_config_json,
                            ..AppSettings::default()
                        });
                        self.app.save_settings()?;

                        // Change to normal mode
                        self.app.change_mode(Mode::Normal);
                        break;
                    } else {
                        log::error!("Failed to connect to Wi-Fi");

                        let display = &mut self.app.display;

                        Text::new(
                            &format!("WiFi connect failed: {}", self.wifi_client_config.ssid),
                            Point::new(30, 80),
                            text_style,
                        )
                        .draw(display)?;

                        // Update and display the frame
                        self.app.update_and_display()?;
                    }
                }
                Err(RecvTimeoutError::Timeout) => self.app.board.led.toggle()?,
                _ => {}
            }
        }

        Ok(())
    }
}

impl AppMode for InitializeMode {
    fn new(app: App) -> Self {
        let wifi_ap_config = wifi::AccessPointConfiguration {
            ssid: format!("smart-calendar-{}", app.config.device_id)
                .as_str()
                .try_into()
                .unwrap(),
            password: "12345678".try_into().unwrap(),
            auth_method: wifi::AuthMethod::WPA2Personal,
            ..Default::default()
        };

        let wifi_client_config = wifi::ClientConfiguration {
            ..Default::default()
        };

        InitializeMode {
            app,
            wifi_ap_config,
            wifi_client_config,
        }
    }

    fn run(&mut self) {
        if let Err(e) = self.run_internal() {
            self.app.handle_error(e);
        }
    }

    fn retrieve(self) -> App {
        self.app
    }
}

pub struct NormalMode {
    app: App,
}

impl NormalMode {
    fn initialize_wifi(&mut self) -> anyhow::Result<()> {
        self.app
            .board
            .wifi
            .set_configuration(&wifi::Configuration::Client(
                self.app
                    .settings
                    .as_ref()
                    .unwrap()
                    .wifi_config
                    .to_client_config(),
            ))?;

        // Start Wifi
        self.app.board.wifi.start()?;

        // Connect Wifi
        self.app.board.wifi.connect()?;

        // Wait until the network interface is up
        self.app.board.wifi.wait_netif_up()?;

        Ok(())
    }

    fn initialize(&mut self) -> anyhow::Result<()> {
        log::info!("NormalMode::initialize");
        self.initialize_wifi()?;
        self.app.sync_ntp()?;
        Ok(())
    }

    fn run_internal(&mut self) -> anyhow::Result<()> {
        // Initialize the app mode
        self.initialize()?;

        // Get the current time
        let now_local = get_time()
            .with_timezone(&self.app.settings.as_ref().unwrap().timezone)
            .naive_local();
        let weekday = format!("{}", now_local.format("%A"));

        // Create a new main page
        let mut main_page = MainPage::new(now_local);

        // Set weekday
        main_page.set_weekday(weekday);

        // Render the main page
        main_page.draw(&mut self.app.display)?;

        // Update and display the frame
        self.app.update_and_display()?;

        // Sleep for a while
        self.app
            .sleep(self.app.settings.as_ref().unwrap().refresh_interval)?;

        Ok(())
    }
}

impl AppMode for NormalMode {
    fn new(app: App) -> Self {
        Self { app }
    }

    fn run(&mut self) {
        if let Err(e) = self.run_internal() {
            self.app.handle_error(e);
        }
    }

    fn retrieve(self) -> App {
        self.app
    }
}

pub struct App {
    mode: Mode,
    config: AppConfig,
    settings: Option<AppSettings>,

    board: Board,
    sysloop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,

    nvs_storage: EspNvs<NvsDefault>,

    display: VarDisplay<'static, Color>,
}

impl App {
    pub fn new(board: Board, sysloop: EspSystemEventLoop, nvs: EspDefaultNvsPartition) -> Self {
        let nvs_namespace = "calendar";
        let nvs_storage = match EspNvs::new(nvs.clone(), nvs_namespace, true) {
            Ok(nvs) => {
                log::info!("NVS namespace: {}", nvs_namespace);
                nvs
            }
            Err(e) => panic!("Could't get namespace {:?}", e),
        };

        Self {
            mode: Mode::Normal,
            config: AppConfig {
                device_id: const_random!(u8), // Random device ID
            },
            settings: None,
            board,
            sysloop,
            nvs,
            nvs_storage,
            display: create_display().unwrap(),
        }
    }

    fn load_settings(&mut self) -> anyhow::Result<()> {
        let mut buf = [0u8; 1024];
        let str = self.nvs_storage.get_str("settings", &mut buf)?;
        self.settings = match str {
            Some(str) => {
                let settings: AppSettings = serde_json::from_str(str)?;
                Some(settings)
            }
            None => None,
        };

        Ok(())
    }

    fn save_settings(&mut self) -> anyhow::Result<()> {
        let buf = serde_json::to_string(&self.settings)?;
        self.nvs_storage.set_str("settings", &buf)?;

        Ok(())
    }

    fn clear_settings(&mut self) -> anyhow::Result<()> {
        self.nvs_storage.remove("settings")?;
        Ok(())
    }

    fn initialize(&mut self) -> anyhow::Result<()> {
        log::info!("initialize");

        // Turn on the LED
        self.board.led.set_high()?;

        let wakeup_reason = reset::WakeupReason::get();
        match wakeup_reason {
            reset::WakeupReason::Unknown | reset::WakeupReason::Timer => self.mode = Mode::Normal,
            _ => log::info!("Wakeup: {:?}", wakeup_reason),
        }

        // Load settings from NVS
        self.load_settings()?;

        // Set mode to Initialize if settings are not available
        if self.settings.is_none() {
            self.mode = Mode::Initialize;
        }

        // Force initialize if the firmware environment variable is set
        if env!("FORCE_INITIALIZE") == "true" {
            log::info!("Force initialize");
            self.clear_settings()?;
            self.mode = Mode::Initialize;
        }

        log::info!("Settings: {:#?}", self.settings);

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

    fn sync_ntp(&mut self) -> anyhow::Result<()> {
        log::info!("Synchronizing with NTP Server");
        let ntp = EspSntp::new_default()?;
        // Wait until the time is synchronized
        while ntp.get_sync_status() != SyncStatus::Completed {}
        log::info!("Time Sync Completed");
        Ok(())
    }

    fn change_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    // Show error message on the screen
    fn handle_error(&mut self, e: anyhow::Error) {
        log::error!("Unexpected error: {:?}", e);

        // Create a new character style
        let style = MonoTextStyle::new(&FONT_10X20, Black);

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

    pub fn run(mut self) {
        if let Err(e) = self.initialize() {
            self.handle_error(e);
        };

        loop {
            // TODO: Optimize the implementation
            self = match self.mode {
                Mode::Initialize => {
                    let mut app_mode = InitializeMode::new(self);
                    app_mode.run();
                    app_mode.retrieve()
                }
                Mode::Normal => {
                    let mut app_mode = NormalMode::new(self);
                    app_mode.run();
                    app_mode.retrieve()
                }
                _ => unimplemented!(),
            };
        }
    }
}
