use embedded_graphics::prelude::DrawTarget;
use epd_waveshare::color::Color;
use std::fmt::Debug;
use u8g2_fonts::{types::HorizontalAlignment, FontRenderer};

use crate::{common::truncate_string_unicode, draw::DrawError, font, text::Text, Black, GRAY_LUMA};

pub struct Activity {
    name: String,
    days_remaining: i32,
}

impl Activity {
    pub fn new(name: &str, days_remaining: i32) -> Self {
        Self {
            name: name.to_string(),
            days_remaining,
        }
    }
}

pub fn draw_activity<Display>(
    display: &mut Display,
    x: i32,
    y: i32,
    activities: &Vec<Activity>,
) -> Result<(), DrawError>
where
    Display: DrawTarget<Color = Color>,
    Display::Error: Debug,
{
    const ACTIVITY_SPACING: i32 = 33;
    const ACTIVITY_MAX_LEN_DISPLAY: usize = 9;
    const ACTIVITY_MAX_NAME_LEN: usize = 20;

    let activities = if activities.len() > 3 {
        activities
            .iter()
            .take(ACTIVITY_MAX_LEN_DISPLAY)
            .collect::<Vec<&Activity>>()
    } else {
        activities.iter().collect::<Vec<&Activity>>()
    };

    let title_font = FontRenderer::new::<font::inter_bold_32_32>();
    let content_font = FontRenderer::new::<font::noto_sans_tc_semi_bold_16_16>();

    Text::new("Activity", &title_font)
        .x(x)
        .y(y)
        .draw(display, Black)?;

    let mut activity_y = y + 51;
    for activity in activities {
        // Create the text object for the activity name
        let name_text = Text::new(
            &truncate_string_unicode(&activity.name, ACTIVITY_MAX_NAME_LEN),
            &content_font,
        )
        .x(x)
        .y(activity_y);

        // Draw the activity name
        if activity.days_remaining == 0 {
            name_text.draw(display, Black)?;
        } else {
            name_text.draw_gray(display, GRAY_LUMA)?;
        }

        // If the activity is today, display "Today", otherwise display the number of days remaining
        let days_remaining_text = if activity.days_remaining == 0 {
            "Today"
        } else {
            &format!("{} days", &activity.days_remaining)
        };

        // Create the text object for the days remaining
        let days_text = Text::new(days_remaining_text, &content_font)
            .x(x + 233)
            .y(activity_y)
            .horizontal_align(HorizontalAlignment::Right);

        // If the activity is today, draw it in black, otherwise draw it in gray
        if activity.days_remaining == 0 {
            days_text.draw(display, Black)?;
        } else {
            days_text.draw_gray(display, GRAY_LUMA)?;
        }

        activity_y += ACTIVITY_SPACING;
    }

    Ok(())
}
