use embedded_graphics::prelude::DrawTarget;
use epd_waveshare::color::Color;
use std::fmt::Debug;
use u8g2_fonts::FontRenderer;

use crate::{
    draw::{draw_text, DrawError},
    font,
};

pub fn draw_weekday<Display>(
    display: &mut Display,
    weekday: &str,
    x: i32,
    y: i32,
) -> Result<(), DrawError>
where
    Display: DrawTarget<Color = Color>,
    Display::Error: Debug,
{
    let font = FontRenderer::new::<font::inter_bold_48_48>();

    draw_text(display, weekday, font, x, y)?;

    Ok(())
}
