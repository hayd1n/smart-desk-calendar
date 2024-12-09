use chrono::{DateTime, Utc};

use embedded_svc::http::client::Client;
use esp_idf_svc::{http::client::EspHttpConnection, io::Read};
use ics_parser::{Event, IcsParser};

pub struct IcsDownloader<'a> {
    http_client: &'a mut Client<EspHttpConnection>,
    url: String,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
}

impl<'a> IcsDownloader<'a> {
    pub fn new(
        http_client: &'a mut Client<EspHttpConnection>,
        url: &str,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            http_client,
            url: url.to_string(),
            start_date,
            end_date,
        }
    }

    pub fn download_and_parse_ics(&mut self) -> anyhow::Result<Vec<Event>> {
        let url = self.url.clone();
        let request = self.http_client.get(&url)?;
        let response = request.submit()?;

        if (200..=299).contains(&response.status()) {
            let mut buf = [0_u8; 256];
            let mut reader = response;
            let mut offset = 0;
            let mut leftover = String::new();

            let mut parser = IcsParser::new(self.start_date, self.end_date);

            loop {
                if let Ok(size) = Read::read(&mut reader, &mut buf[offset..]) {
                    if size == 0 {
                        break;
                    }
                    let size_plus_offset = size + offset;
                    let text = match std::str::from_utf8(&buf[..size_plus_offset]) {
                        Ok(text) => {
                            leftover.push_str(text);
                            std::mem::take(&mut leftover)
                        }
                        Err(error) => {
                            let valid_up_to = error.valid_up_to();
                            unsafe {
                                leftover
                                    .push_str(std::str::from_utf8_unchecked(&buf[..valid_up_to]));
                            }
                            buf.copy_within(valid_up_to.., 0);
                            offset = size_plus_offset - valid_up_to;
                            continue;
                        }
                    };

                    // self.parse_ics_chunk(&mut current_event, &text, &mut leftover);
                    parser.parse_ics_chunk(&text);
                    offset = 0;
                }
            }

            return Ok(parser.get_events());
        } else {
            // eprintln!("Unexpected response code: {}", response.status());
            return Err(anyhow::anyhow!(
                "Unexpected response code: {}",
                response.status()
            ));
        }
    }
}
