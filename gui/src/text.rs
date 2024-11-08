use embedded_graphics::{
    pixelcolor::Gray8,
    prelude::{DrawTarget, Point, Size},
    primitives::Rectangle,
};
use epd_waveshare::color::Color;
use std::fmt::Debug;
use u8g2_fonts::{
    types::{FontColor, HorizontalAlignment, VerticalPosition},
    FontRenderer,
};

use crate::{
    display::FakeDisplay,
    draw::{floyd_steinberg_dither, DrawError},
};

pub struct Text {
    text: String,
    font: FontRenderer,
    x: i32,
    y: i32,
    vertical_pos: VerticalPosition,
    horizontal_align: HorizontalAlignment,
}

impl Text {
    pub fn new(text: &str, font: &FontRenderer) -> Self {
        Self {
            text: text.to_string(),
            font: font.clone(),
            x: 0,
            y: 0,
            vertical_pos: VerticalPosition::Top,
            horizontal_align: HorizontalAlignment::Left,
        }
    }

    pub fn text(mut self, text: &str) -> Self {
        self.text = text.to_string();
        self
    }

    pub fn font(mut self, font: &FontRenderer) -> Self {
        self.font = font.clone();
        self
    }

    pub fn x(mut self, x: i32) -> Self {
        self.x = x;
        self
    }

    pub fn y(mut self, y: i32) -> Self {
        self.y = y;
        self
    }

    pub fn vertical_pos(mut self, vertical_pos: VerticalPosition) -> Self {
        self.vertical_pos = vertical_pos;
        self
    }

    pub fn horizontal_align(mut self, horizontal_align: HorizontalAlignment) -> Self {
        self.horizontal_align = horizontal_align;
        self
    }

    pub fn bounding_box(&self) -> Result<Rectangle, DrawError> {
        let position = Point::new(self.x, self.y);

        // Get the bounding box of the text to determine the width and height
        self.font
            .get_rendered_dimensions_aligned(
                self.text.as_str(),
                position,
                self.vertical_pos,
                self.horizontal_align,
            )
            .map_err(|err| DrawError::DrawFailed(format!("{:?}", err)))?
            .ok_or(DrawError::DrawFailed(
                "Failed to get bounding box".to_string(),
            ))
    }

    pub fn draw<Display>(&self, display: &mut Display, color: Color) -> Result<Rectangle, DrawError>
    where
        Display: DrawTarget<Color = Color>,
        Display::Error: Debug,
    {
        let position = Point::new(self.x, self.y);

        // Get the bounding box of the text to determine the width and height
        let bounding_box = self
            .font
            .get_rendered_dimensions_aligned(
                self.text.as_str(),
                position,
                self.vertical_pos,
                self.horizontal_align,
            )
            .map_err(|err| DrawError::DrawFailed(format!("{:?}", err)))?
            .unwrap();

        // Render the text on the temporary display
        self.font
            .render_aligned(
                self.text.as_str(),
                position,
                self.vertical_pos,
                self.horizontal_align,
                FontColor::Transparent(color),
                display,
            )
            .map_err(|err| DrawError::DrawFailed(format!("{:?}", err)))?;

        Ok(bounding_box)
    }

    pub fn draw_gray<Display>(
        &self,
        display: &mut Display,
        luma: u8,
    ) -> Result<Rectangle, DrawError>
    where
        Display: DrawTarget<Color = Color>,
        Display::Error: Debug,
    {
        let position = Point::new(0, 0);

        let offset = {
            // Caculate the position offset
            let bounding_box = self
                .font
                .get_rendered_dimensions_aligned(
                    self.text.as_str(),
                    position,
                    VerticalPosition::Top,
                    HorizontalAlignment::Left,
                )
                .map_err(|err| DrawError::DrawFailed(format!("{:?}", err)))?
                .unwrap();

            Point::new(bounding_box.top_left.x, bounding_box.top_left.y)
        };

        // Get the bounding box of the text to determine the width and height
        let bounding_box = self
            .font
            .get_rendered_dimensions_aligned(
                self.text.as_str(),
                position,
                self.vertical_pos,
                self.horizontal_align,
            )
            .map_err(|err| DrawError::DrawFailed(format!("{:?}", err)))?
            .unwrap();

        // Calculate the real width and height of the text
        let real_width = (bounding_box.size.width as i32 + offset.x) as u32;
        let real_height = (bounding_box.size.height as i32 + offset.y) as u32;

        // Create a temporary display to render the text
        let mut gray_display: FakeDisplay<Gray8> =
            FakeDisplay::new(Size::new(real_width, real_height));

        // Render the text on the temporary display
        self.font
            .render_aligned(
                self.text.as_str(),
                position,
                VerticalPosition::Top,
                HorizontalAlignment::Left,
                FontColor::Transparent(Gray8::new(luma)),
                &mut gray_display,
            )
            .map_err(|err| DrawError::DrawFailed(format!("{:?}", err)))?;

        let top_left = bounding_box.top_left;
        let real_x = self.x + top_left.x - offset.x;
        let real_y = self.y + top_left.y - offset.y;

        // Convert the temporary display to a binary display using Floyd-Steinberg dithering
        floyd_steinberg_dither(
            &mut gray_display,
            display,
            real_x,
            real_y,
            real_width,
            real_height,
        );

        Ok(bounding_box)
    }
}
