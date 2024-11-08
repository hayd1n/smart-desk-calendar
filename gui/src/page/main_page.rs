use chrono::NaiveDateTime;
use embedded_graphics::prelude::DrawTarget;
use epd_waveshare::color::Color::{self};
use std::fmt::Debug;

use crate::{
    components::{activity::Activity, draw_activity, draw_calendar, draw_date, draw_weekday},
    draw::{clear, DrawError},
};

pub struct MainPage {
    pub weekday: String,
    pub now: NaiveDateTime,
}

impl MainPage {
    pub fn new(now: NaiveDateTime) -> Self {
        Self {
            now,
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
        let date = self.now.date();

        // Clear the display
        clear(display)?;

        // Draw the weekday component
        draw_weekday(display, &self.weekday, 35, 40)?;

        // Draw the date component
        draw_date(display, 766, 40, date)?;

        // Draw the calendar component
        draw_calendar(display, 35, 121, date)?;

        let activities = vec![
            Activity::new("Culture Exchange Activity", 0),
            Activity::new("DS HW 2 deadline", 4),
            Activity::new("English presentation", 7),
        ];

        // Draw the activity component
        draw_activity(display, 533, 121, &activities)?;

        Ok(())
    }
}
