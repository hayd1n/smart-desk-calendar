use chrono::{NaiveDate, NaiveDateTime};
use embedded_graphics::prelude::DrawTarget;
use epd_waveshare::color::Color::{self};
use std::fmt::Debug;

use crate::{
    components::{
        activity::DaysRemaining, draw_activity, draw_calendar, draw_date, draw_small_clock,
        draw_weekday,
    },
    draw::{clear, DrawError},
};

pub struct Event {
    name: String,
    date: NaiveDate,
}

impl Event {
    pub fn new(name: &str, date: NaiveDate) -> Self {
        Self {
            name: name.to_string(),
            date,
        }
    }
}

pub struct MainPage {
    pub weekday: String,
    pub now: NaiveDateTime,
    pub events: Vec<Event>,
}

impl MainPage {
    pub fn new(now: NaiveDateTime) -> Self {
        Self {
            now,
            weekday: String::new(),
            events: vec![],
        }
    }

    pub fn set_weekday(&mut self, weekday: String) {
        self.weekday = weekday;
    }

    pub fn set_events(&mut self, events: Vec<Event>) {
        self.events = events;
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

        let events_date = self
            .events
            .iter()
            .map(|event| event.date)
            .collect::<Vec<NaiveDate>>();

        // Draw the calendar component
        draw_calendar(display, 35, 121, date, &events_date)?;

        let days_remaining = self
            .events
            .iter()
            .filter_map(|event| {
                let days_remaining = event.date.signed_duration_since(date).num_days();
                if days_remaining >= 0 {
                    Some(DaysRemaining::new(
                        &event.name,
                        days_remaining.try_into().unwrap(),
                    ))
                } else {
                    None
                }
            })
            .collect();

        // Draw the activity component
        draw_activity(display, 533, 121, &days_remaining)?;

        Ok(())
    }
}
