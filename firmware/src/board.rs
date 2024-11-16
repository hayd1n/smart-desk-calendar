use epd_waveshare::{epd7in5_v2::Epd7in5, prelude::WaveshareDisplay};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        delay::Delay,
        gpio::{self, AnyIOPin, Input, Output, PinDriver},
        prelude::*,
        spi::{self, SpiDeviceDriver, SpiDriver},
    },
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};

pub struct Board {
    pub led: PinDriver<'static, gpio::Gpio2, Output>,

    pub spi: SpiDeviceDriver<'static, SpiDriver<'static>>,
    pub pwr: PinDriver<'static, gpio::Gpio23, Output>,

    pub delay: Delay,

    pub epd: Epd7in5<
        SpiDeviceDriver<'static, SpiDriver<'static>>,
        PinDriver<'static, gpio::Gpio25, Input>,
        PinDriver<'static, gpio::Gpio27, Output>,
        PinDriver<'static, gpio::Gpio26, Output>,
        Delay,
    >,

    pub wifi: BlockingWifi<EspWifi<'static>>,
}

impl Board {
    pub fn init(
        peripherals: Peripherals,
        sysloop: EspSystemEventLoop,
        nvs: EspDefaultNvsPartition,
    ) -> Self {
        let spi_p_pin = peripherals.spi3;
        let sclk_pin = peripherals.pins.gpio13;
        let sdo_pin = peripherals.pins.gpio14;
        let cs_pin = peripherals.pins.gpio15;

        let busy_in_pin = peripherals.pins.gpio25;
        let dc_pin = peripherals.pins.gpio27;
        let rst_pin = peripherals.pins.gpio26;

        let pwr_pin = peripherals.pins.gpio23;

        let led_pin = peripherals.pins.gpio2;

        let led = PinDriver::output(led_pin).unwrap();

        let mut spi = SpiDeviceDriver::new_single(
            spi_p_pin,
            sclk_pin,
            sdo_pin,
            Option::<AnyIOPin>::None,
            Some(cs_pin),
            &spi::config::DriverConfig::new(),
            &spi::config::Config::new().baudrate(10.MHz().into()),
        )
        .unwrap();

        let busy_in = PinDriver::input(busy_in_pin).unwrap();
        let dc = PinDriver::output(dc_pin).unwrap();
        let rst = PinDriver::output(rst_pin).unwrap();

        let mut pwr = PinDriver::output(pwr_pin).unwrap();

        // Turn on E-Ink display
        pwr.set_high().unwrap();

        // Setup delay
        let mut delay = Delay::new_default();

        // Setup EPD
        let epd = Epd7in5::new(&mut spi, busy_in, dc, rst, &mut delay, None).unwrap();

        let esp_wifi = EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs)).unwrap();
        let wifi = BlockingWifi::wrap(esp_wifi, sysloop.clone()).unwrap();

        Self {
            led,
            spi,
            pwr,
            delay,
            epd,
            wifi,
        }
    }
}
