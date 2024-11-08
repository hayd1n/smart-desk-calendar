use chrono::{Datelike, NaiveDate};
use embedded_graphics::Drawable;
use embedded_graphics::{
    prelude::{DrawTarget, Point, Primitive},
    primitives::{Circle, PrimitiveStyleBuilder},
};
use epd_waveshare::color::Color;
use std::fmt::Debug;
use u8g2_fonts::FontRenderer;

use crate::{draw::DrawError, font, text::Text, Black, White, GRAY_LUMA};

const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

fn last_day_of_month(year: i32, month: u32) -> chrono::NaiveDate {
    use chrono::NaiveDate;
    // 生成下一个月的第一天，然后减去一天
    let next_month_first = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    };
    next_month_first - chrono::Duration::days(1)
}

pub fn draw_calendar<Display>(
    display: &mut Display,
    x: i32,
    y: i32,
    date: NaiveDate,
) -> Result<(), DrawError>
where
    Display: DrawTarget<Color = Color>,
    Display::Error: Debug,
{
    const CALENDAR_WIDTH: i32 = 438;
    const Y_SPACING: i32 = 52;

    let year = date.year();
    let month = date.month();

    let today_weekday_num = date.weekday().num_days_from_sunday();

    // Find the first day of the month
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let last_day = last_day_of_month(year, month);

    // Get the day of the week that the first day is
    let start_weekday = first_day.weekday();
    let start_weekday_num = start_weekday.num_days_from_sunday();

    let font_bold_16 = FontRenderer::new::<font::inter_bold_16_16>();
    let font_bold_32 = FontRenderer::new::<font::inter_bold_32_32>();

    let month_text = date.format("%B").to_string();

    Text::new(&month_text, &font_bold_32)
        .x(x)
        .y(y)
        .draw(display, Black)?;

    Text::new(&year.to_string(), &font_bold_32)
        .x(x + CALENDAR_WIDTH)
        .y(y)
        .horizontal_align(u8g2_fonts::types::HorizontalAlignment::Right)
        .draw_gray(display, GRAY_LUMA)?;

    let first_weekday = Text::new(WEEKDAYS.first().unwrap(), &font_bold_16)
        .x(x)
        .y(y + 48)
        .horizontal_align(u8g2_fonts::types::HorizontalAlignment::Left);

    let last_weekday = Text::new(WEEKDAYS.last().unwrap(), &font_bold_16)
        .x(x + CALENDAR_WIDTH)
        .y(y + 48)
        .horizontal_align(u8g2_fonts::types::HorizontalAlignment::Right);

    let first_weekday_box = first_weekday.bounding_box()?;
    let last_weekday_box = last_weekday.bounding_box()?;

    let first_weekday_box_center_x =
        first_weekday_box.top_left.x + ((first_weekday_box.size.width / 2) as i32);
    let last_weekday_box_center_x =
        last_weekday_box.top_left.x + ((last_weekday_box.size.width / 2) as i32);

    let first_weekday_box_center_y =
        first_weekday_box.top_left.y + ((first_weekday_box.size.height / 2) as i32);

    let weekday_spacing = {
        let width = last_weekday_box_center_x - first_weekday_box_center_x;
        width / ((WEEKDAYS.len() as i32) - 1)
    };

    // Draw the weekdays
    {
        // Create a vector of Text objects for each weekday
        let mut weekdays_text = Vec::new();
        weekdays_text.push(first_weekday);
        for i in 1..(WEEKDAYS.len() - 1) {
            weekdays_text.push(
                Text::new(WEEKDAYS[i], &font_bold_16)
                    .x(first_weekday_box_center_x + ((i as i32) * weekday_spacing))
                    .y(y + 48)
                    .horizontal_align(u8g2_fonts::types::HorizontalAlignment::Center),
            );
        }
        weekdays_text.push(last_weekday);

        // Draw the weekdays
        for (i, weekday_text) in weekdays_text.iter().enumerate() {
            if i == (today_weekday_num as usize) {
                weekday_text.draw(display, Black)?;
            } else {
                weekday_text.draw_gray(display, GRAY_LUMA)?;
            }
        }
    }

    // Draw the days of the month
    {
        let mut pos: i32 = start_weekday_num.try_into().unwrap();
        // Traverse each day of the month
        let mut day = first_day;
        while day <= last_day {
            let row = pos / 7;
            let col = pos % 7;

            let day_x = first_weekday_box_center_x + (col * weekday_spacing);
            let day_y = first_weekday_box_center_y + Y_SPACING + (Y_SPACING * row);

            let text = Text::new(&day.day().to_string(), &font_bold_16)
                .x(day_x)
                .y(day_y)
                .vertical_pos(u8g2_fonts::types::VerticalPosition::Center)
                .horizontal_align(u8g2_fonts::types::HorizontalAlignment::Center);

            if day == date {
                let diameter = 34;
                let radius = diameter / 2;

                let style = PrimitiveStyleBuilder::new().fill_color(Black).build();

                Circle::new(
                    Point::new(
                        day_x - TryInto::<i32>::try_into(radius).unwrap(),
                        day_y - TryInto::<i32>::try_into(radius).unwrap(),
                    ),
                    diameter,
                )
                .into_styled(style)
                .draw(display)
                .map_err(|err| DrawError::DrawFailed(format!("{:?}", err)))?;

                text.draw(display, White)?;
            } else {
                text.draw_gray(display, GRAY_LUMA)?;
            }

            // Move to next date
            day = day.succ_opt().unwrap();
            pos += 1;
        }
    }

    Ok(())
}
