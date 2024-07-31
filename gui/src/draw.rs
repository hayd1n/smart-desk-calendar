use embedded_graphics::{
    pixelcolor::Gray8,
    prelude::{DrawTarget, IntoStorage, Point, Size},
    primitives::Rectangle,
    Drawable, Pixel,
};
use epd_waveshare::color::Color::{self, Black as White, White as Black};
use std::fmt::Debug;
use thiserror::Error;
use u8g2_fonts::{
    types::{FontColor, HorizontalAlignment, VerticalPosition},
    FontRenderer,
};

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

pub fn draw_text<Display>(
    display: &mut Display,
    text: &str,
    font: FontRenderer,
    x: i32,
    y: i32,
) -> Result<Rectangle, DrawError>
where
    Display: DrawTarget<Color = Color>,
    Display::Error: Debug,
{
    let position = Point::new(x, y);
    let vertical_pos = VerticalPosition::Top;
    let horizontal_align = HorizontalAlignment::Left;

    // Get the bounding box of the text to determine the width and height
    let bounding_box = font
        .get_rendered_dimensions_aligned(text, position, vertical_pos, horizontal_align)
        .map_err(|err| DrawError::DrawFailed(format!("{:?}", err)))?
        .unwrap();

    // Render the text on the temporary display
    font.render_aligned(
        text,
        position,
        vertical_pos,
        horizontal_align,
        FontColor::Transparent(Black),
        display,
    )
    .map_err(|err| DrawError::DrawFailed(format!("{:?}", err)))?;

    Ok(bounding_box)
}

pub fn draw_text_gray<Display>(
    display: &mut Display,
    text: &str,
    font: FontRenderer,
    luma: u8,
    x: i32,
    y: i32,
) -> Result<Rectangle, DrawError>
where
    Display: DrawTarget<Color = Color>,
    Display::Error: Debug,
{
    let position = Point::new(0, 0);
    let vertical_pos = VerticalPosition::Top;
    let horizontal_align = HorizontalAlignment::Left;

    // Get the bounding box of the text to determine the width and height
    let bounding_box = font
        .get_rendered_dimensions_aligned(text, position, vertical_pos, horizontal_align)
        .map_err(|err| DrawError::DrawFailed(format!("{:?}", err)))?
        .unwrap();

    let top_left = bounding_box.top_left;

    // Calculate the real width and height of the text
    let real_width = (bounding_box.size.width as i32 + top_left.x) as u32;
    let real_height = (bounding_box.size.height as i32 + top_left.y) as u32;

    // Create a temporary display to render the text
    let mut gray_display: FakeDisplay<Gray8> = FakeDisplay::new(Size::new(real_width, real_height));

    // Render the text on the temporary display
    font.render_aligned(
        text,
        position,
        vertical_pos,
        horizontal_align,
        FontColor::Transparent(Gray8::new(luma)),
        &mut gray_display,
    )
    .map_err(|err| DrawError::DrawFailed(format!("{:?}", err)))?;

    // Convert the temporary display to a binary display using Floyd-Steinberg dithering
    floyd_steinberg_dither(
        &mut gray_display,
        display,
        x + top_left.x,
        y + top_left.y,
        real_width,
        real_height,
    );

    Ok(bounding_box)
}

#[derive(Debug, Error)]
pub enum DrawError {
    #[error("Draw failed: {0}")]
    DrawFailed(String),
}
