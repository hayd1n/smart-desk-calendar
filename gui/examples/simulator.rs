use chrono::Local;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use epd_waveshare::{
    color::Color::{self},
    epd7in5_v2::{HEIGHT, WIDTH},
};
use gui::page::main_page::MainPage;

fn main() -> anyhow::Result<()> {
    let mut display: SimulatorDisplay<Color> = SimulatorDisplay::new(Size::new(
        WIDTH.try_into().unwrap(),
        HEIGHT.try_into().unwrap(),
    ));

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::Inverted)
        .scale(1)
        .pixel_spacing(0)
        .build();
    let mut window = Window::new("Simulator", &output_settings);

    let mut main_page = MainPage::new();

    'running: loop {
        let now = Local::now().naive_local();
        let weekday = format!("{}", now.format("%A"));

        main_page.set_weekday(weekday);

        main_page.draw(&mut display)?;

        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                _ => {}
            }
        }
    }

    Ok(())
}
