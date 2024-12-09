use embedded_graphics::{
    pixelcolor::Gray8,
    prelude::{DrawTarget, Point, Primitive, Size},
    primitives::PrimitiveStyleBuilder,
    Drawable,
};
use epd_waveshare::color::Color;
use std::fmt::Debug;

use crate::{
    display::FakeDisplay,
    draw::{floyd_steinberg_dither, DrawError},
};

pub struct Circle {
    pub x: i32,
    pub y: i32,
    pub diameter: u32,
}

impl Circle {
    pub fn new(x: i32, y: i32, diameter: u32) -> Self {
        Self { x, y, diameter }
    }

    pub fn x(mut self, x: i32) -> Self {
        self.x = x;
        self
    }

    pub fn y(mut self, y: i32) -> Self {
        self.y = y;
        self
    }

    pub fn diameter(mut self, diameter: u32) -> Self {
        self.diameter = diameter;
        self
    }

    pub fn draw<Display>(&self, display: &mut Display, color: Color) -> Result<(), DrawError>
    where
        Display: DrawTarget<Color = Color>,
        Display::Error: Debug,
    {
        let style = PrimitiveStyleBuilder::new().fill_color(color).build();
        let radius = self.diameter / 2;
        embedded_graphics::primitives::Circle::new(
            Point::new(
                self.x - TryInto::<i32>::try_into(radius).unwrap(),
                self.y - TryInto::<i32>::try_into(radius).unwrap(),
            ),
            self.diameter,
        )
        .into_styled(style)
        .draw(display)
        .map_err(|err| DrawError::DrawFailed(format!("{:?}", err)))?;

        Ok(())
    }

    pub fn draw_gray<Display>(&self, display: &mut Display, luma: u8) -> Result<(), DrawError>
    where
        Display: DrawTarget<Color = Color>,
        Display::Error: Debug,
    {
        let mut gray_display: FakeDisplay<Gray8> =
            FakeDisplay::new(Size::new(self.diameter, self.diameter));

        let style = PrimitiveStyleBuilder::new()
            .fill_color(Gray8::new(luma))
            .build();

        embedded_graphics::primitives::Circle::new(Point::new(0, 0), self.diameter)
            .into_styled(style)
            .draw(&mut gray_display)
            .map_err(|err| DrawError::DrawFailed(format!("{:?}", err)))?;

        let radius = self.diameter as i32 / 2;

        // Convert the temporary display to a binary display using Floyd-Steinberg dithering
        floyd_steinberg_dither(
            &mut gray_display,
            display,
            self.x - radius,
            self.y - radius,
            self.diameter,
            self.diameter,
        );

        Ok(())
    }
}
