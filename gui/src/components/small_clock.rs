use chrono::NaiveTime;
use embedded_graphics::prelude::DrawTarget;
use epd_waveshare::color::Color;
use std::fmt::Debug;
use u8g2_fonts::{types::HorizontalAlignment, FontRenderer};

use crate::{draw::DrawError, font, text::Text, Black};

pub fn draw_small_clock<Display>(
    display: &mut Display,
    x: i32,
    y: i32,
    time: NaiveTime,
) -> Result<(), DrawError>
where
    Display: DrawTarget<Color = Color>,
    Display::Error: Debug,
{
    let font = FontRenderer::new::<font::inter_bold_16_16>();

    Text::new(&time.format("%_H:%M").to_string(), &font)
        .x(x)
        .y(y)
        .horizontal_align(HorizontalAlignment::Right)
        .draw(display, Black)?;

    Ok(())
}
