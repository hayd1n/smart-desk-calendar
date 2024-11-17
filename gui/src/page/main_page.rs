use chrono::NaiveDateTime;
use embedded_graphics::prelude::DrawTarget;
use epd_waveshare::color::Color::{self};
use std::fmt::Debug;

use crate::{
    components::{
        activity::Activity, draw_activity, draw_calendar, draw_date, draw_small_clock, draw_weekday,
    },
    draw::{clear, DrawError},
};

pub struct MainPage {
    pub weekday: String,
    pub now: NaiveDateTime,
    pub activities: Vec<Activity>,
}

impl MainPage {
    pub fn new(now: NaiveDateTime) -> Self {
        Self {
            now,
            weekday: String::new(),
            activities: vec![],
        }
    }

    pub fn set_weekday(&mut self, weekday: String) {
        self.weekday = weekday;
    }

    pub fn set_activities(&mut self, activities: Vec<Activity>) {
        self.activities = activities;
    }

    pub fn draw<Display>(&self, display: &mut Display) -> Result<(), DrawError>
    where
        Display: DrawTarget<Color = Color>,
        Display::Error: Debug,
    {
        let date = self.now.date();

        // Clear the display
        clear(display)?;

        // Draw the small clock component
        draw_small_clock(display, 766, 18, self.now.time())?;

        // Draw the weekday component
        draw_weekday(display, 35, 40, &self.weekday)?;

        // Draw the date component
        draw_date(display, 766, 40, date)?;

        // Draw the calendar component
        draw_calendar(display, 35, 121, date)?;

        // Draw the activity component
        draw_activity(display, 533, 121, &self.activities)?;

        Ok(())
    }
}
