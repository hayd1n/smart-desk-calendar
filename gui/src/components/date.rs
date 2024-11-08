use chrono::{Datelike, NaiveDate};
use embedded_graphics::prelude::DrawTarget;
use epd_waveshare::color::Color;
use std::fmt::Debug;
use u8g2_fonts::{
    types::{HorizontalAlignment, VerticalPosition},
    FontRenderer,
};

use crate::{draw::DrawError, font, text::Text, Black};

pub fn draw_date<Display>(
    display: &mut Display,
    x: i32,
    y: i32,
    date: NaiveDate,
) -> Result<(), DrawError>
where
    Display: DrawTarget<Color = Color>,
    Display::Error: Debug,
{
    let month_short = date.format("%b").to_string();
    let day = date.day().to_string();

    let day_font = FontRenderer::new::<font::inter_bold_48_48>();
    let month_font = FontRenderer::new::<font::inter_bold_32_32>();

    let day_box = Text::new(&day, &day_font)
        .x(x)
        .y(y)
        .horizontal_align(HorizontalAlignment::Right)
        .draw(display, Black)?;

    Text::new(&month_short, &month_font)
        .x(x - (day_box.size.width as i32) - 10)
        .y(y + day_box.size.height as i32)
        .vertical_pos(VerticalPosition::Baseline)
        .horizontal_align(HorizontalAlignment::Right)
        .draw(display, Black)?;

    Ok(())
}
