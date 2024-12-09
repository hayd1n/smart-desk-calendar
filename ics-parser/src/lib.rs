use std::cmp::Ordering;

use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;

#[derive(Debug, Clone)]
struct TemporaryEvent {
    pub summary: String,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub summary: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

// 實作 PartialEq 來比較兩個 Event 是否相等
impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
    }
}

// 實作 Eq trait 表示這個比較是可反射的
impl Eq for Event {}

// 實作 PartialOrd 來定義如何比較兩個 Event
impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // 由近到遠排序，所以用 other 比較 self (降序)
        Some(self.start.cmp(&other.start))
    }
}

// 實作 Ord 來提供完整的排序功能
impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        // 由近到遠排序，所以用 other 比較 self (降序)
        other.start.cmp(&self.start)
    }
}

pub struct IcsParser {
    current_event: TemporaryEvent,
    leftover: String,
    events: Vec<Event>,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
}

impl IcsParser {
    pub fn new(start_date: Option<DateTime<Utc>>, end_date: Option<DateTime<Utc>>) -> Self {
        Self {
            current_event: TemporaryEvent {
                summary: String::new(),
                start: None,
                end: None,
            },
            leftover: String::new(),
            events: Vec::new(),
            start_date,
            end_date,
        }
    }

    // 分析ICS片段內容，並處理跨段的未完成行
    pub fn parse_ics_chunk(&mut self, chunk: &str) {
        self.leftover.push_str(chunk);

        // println!("Leftover: {:?}", self.leftover);

        for line in self.leftover.lines() {
            if line.starts_with("BEGIN:VEVENT") {
                self.current_event = TemporaryEvent {
                    summary: String::new(),
                    start: None,
                    end: None,
                };
            } else if line.starts_with("SUMMARY:") {
                self.current_event.summary = line["SUMMARY:".len()..].to_string();
            } else if line.starts_with("DTSTART") {
                self.current_event.start = parse_datetime(line);
            } else if line.starts_with("DTEND") {
                self.current_event.end = parse_datetime(line);
            } else if line.starts_with("END:VEVENT") {
                // println!("Parsed event: {:?}", self.current_event);
                if let (Some(start_date), Some(end_date)) = (
                    self.current_event.start.as_ref(),
                    self.current_event.end.as_ref(),
                ) {
                    if let (Some(filter_start_date), Some(filter_end_date)) =
                        (self.start_date.as_ref(), self.end_date.as_ref())
                    {
                        if start_date >= filter_start_date && end_date <= filter_end_date {
                            self.events.push(Event {
                                summary: self.current_event.summary.clone(),
                                start: start_date.clone(),
                                end: end_date.clone(),
                            });
                        }
                    } else {
                        self.events.push(Event {
                            summary: self.current_event.summary.clone(),
                            start: start_date.clone(),
                            end: end_date.clone(),
                        });
                    }
                }
            }
        }

        if !self.leftover.ends_with('\n') {
            self.leftover = self.leftover.lines().last().unwrap_or("").to_string();
        } else {
            self.leftover.clear();
        }
    }

    pub fn get_events(self) -> Vec<Event> {
        self.events
    }
}

fn parse_datetime(datetime_str: &str) -> Option<DateTime<Utc>> {
    if let Some((tzid_part, time_part)) = datetime_str.split_once(':') {
        if tzid_part.contains("TZID=") {
            if let Some(tzid) = tzid_part.split('=').nth(1) {
                let timezone: Tz = tzid.parse().ok()?;
                let naive_datetime =
                    NaiveDateTime::parse_from_str(time_part, "%Y%m%dT%H%M%S").ok()?;
                let datetime_with_timezone =
                    timezone.from_local_datetime(&naive_datetime).single()?;
                return Some(datetime_with_timezone.with_timezone(&Utc)); // 使用 with_timezone(&Utc)
            }
        } else if tzid_part.contains("VALUE=") {
            if let Some(date) = tzid_part.split('=').nth(1) {
                if date == "DATE" {
                    let naive_date = NaiveDate::parse_from_str(time_part, "%Y%m%d").ok()?;
                    return Some(naive_date.and_hms_opt(0, 0, 0).unwrap().and_utc());
                } else {
                    return None;
                }
            }
        } else if time_part.ends_with('Z') {
            if let Ok(utc_datetime) = DateTime::parse_from_rfc3339(time_part) {
                return Some(utc_datetime.with_timezone(&Utc));
            } else {
                return iso8601::datetime(time_part)
                    .ok()?
                    .into_naive()
                    .map(|naive| naive.and_utc());
            }
        } else {
            let naive_datetime = NaiveDateTime::parse_from_str(time_part, "%Y%m%dT%H%M%S").ok()?;
            // 這裡假設是 UTC，本地時間可以依應用需求改為其他預設時區
            return Some(naive_datetime.and_utc());
        }
    } else if datetime_str.ends_with('Z') {
        if let Ok(utc_datetime) = DateTime::parse_from_rfc3339(datetime_str) {
            return Some(utc_datetime.with_timezone(&Utc));
        } else {
            return iso8601::datetime(datetime_str)
                .ok()?
                .into_naive()
                .map(|naive| naive.and_utc());
        }
    } else {
        let naive_datetime = NaiveDateTime::parse_from_str(datetime_str, "%Y%m%dT%H%M%S").ok()?;
        // 這裡假設是 UTC，本地時間可以依應用需求改為其他預設時區
        return Some(naive_datetime.and_utc());
    }
    None
}
