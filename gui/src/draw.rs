use embedded_graphics::{
    pixelcolor::Gray8,
    prelude::{DrawTarget, IntoStorage, Point},
    Drawable, Pixel,
};
#[allow(unused_imports)]
use epd_waveshare::color::Color::{self, Black as White, White as Black};
use std::fmt::Debug;
use thiserror::Error;

use crate::display::FakeDisplay;

pub fn floyd_steinberg_dither<Display>(
    gray_display: &mut FakeDisplay<Gray8>,
    binary_display: &mut Display,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) where
    Display: DrawTarget<Color = Color>,
    Display::Error: Debug,
{
    let mut error: i16;
    // let mut quant_error: i16;
    let mut err_buffer: Vec<i16> = vec![0; (width * height) as usize];

    for i_y in 0..height {
        for i_x in 0..width {
            let index = i_y * width + i_x;
            let old_pixel = gray_display
                .get_pixel(Point::new(i_x.try_into().unwrap(), i_y.try_into().unwrap()))
                .into_storage() as i16;
            let new_pixel = if old_pixel + err_buffer[index as usize] < 128 {
                0
            } else {
                255
            };
            let new_pixel_bin = if new_pixel == 0 { White } else { Black };
            // image[index] = new_pixel as u8;
            Pixel(Point::new(i_x as i32 + x, i_y as i32 + y), new_pixel_bin)
                .draw(binary_display)
                .unwrap();
            error = old_pixel - new_pixel;

            if i_x + 1 < width {
                err_buffer[(index + 1) as usize] += error * 7 / 16;
            }
            if i_x > 0 && i_y + 1 < height {
                err_buffer[(index + width - 1) as usize] += error * 3 / 16;
            }
            if i_y + 1 < height {
                err_buffer[(index + width) as usize] += error * 5 / 16;
            }
            if i_x + 1 < width && i_y + 1 < height {
                err_buffer[(index + width + 1) as usize] += error * 1 / 16;
            }
        }
    }
}

pub fn clear<Display>(display: &mut Display) -> Result<(), DrawError>
where
    Display: DrawTarget<Color = Color>,
{
    display
        .clear(White)
        .map_err(|_| DrawError::DrawFailed("Failed to clear display".to_string()))
}

#[derive(Debug, Error)]
pub enum DrawError {
    #[error("Draw failed: {0}")]
    DrawFailed(String),
}
