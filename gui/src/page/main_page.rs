use embedded_graphics::prelude::DrawTarget;
use epd_waveshare::color::Color::{self};
use std::fmt::Debug;

use crate::{
    components::draw_weekday,
    draw::{clear, DrawError},
};

pub struct MainPage {
    pub weekday: String,
}

impl MainPage {
    pub fn new() -> Self {
        Self {
            weekday: String::new(),
        }
    }

    pub fn set_weekday(&mut self, weekday: String) {
        self.weekday = weekday;
    }

    pub fn draw<Display>(&self, display: &mut Display) -> Result<(), DrawError>
    where
        Display: DrawTarget<Color = Color>,
        Display::Error: Debug,
    {
        // Clear the display
        clear(display)?;

        // Draw the weekday component
        draw_weekday(display, &self.weekday, 35, 40)?;

        Ok(())
    }
}
