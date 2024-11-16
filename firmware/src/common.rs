use std::time::SystemTime;

use chrono::{DateTime, NaiveDate, Utc};

pub fn get_last_day_of_month(year: i32, month: u32) -> NaiveDate {
    let next_month_first = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    };
    next_month_first - chrono::Duration::days(1)
}

pub fn get_time() -> DateTime<Utc> {
    // Obtain System Time
    let st_now = SystemTime::now();
    // Convert to UTC Time
    st_now.into()
}
