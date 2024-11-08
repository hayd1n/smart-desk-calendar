use chrono::Local;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay};
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

    let now = Local::now().naive_local().date();
    let weekday = format!("{}", now.format("%A"));

    let mut main_page = MainPage::new(now);

    main_page.set_weekday(weekday);

    main_page.draw(&mut display)?;

    let output_image = display.to_rgb_output_image(&output_settings);

    let path = std::env::args_os().nth(1).unwrap_or("output.png".into());
    output_image.save_png(path).unwrap();

    Ok(())
}
