use chrono::{DateTime, Utc};
use ical::{
    generator::{IcalEvent, IcalEventBuilder, Property},
    ical_property,
};
use notion::models::{
    properties::{DateOrDateTime, PropertyValue},
    Page,
};

#[derive(Debug)]
pub struct Event {
    id: String,
    title: String,
    changed: DateTime<Utc>,
    start: DateOrDateTime,
    end: Option<DateOrDateTime>,
}

impl TryFrom<Page> for Event {
    type Error = anyhow::Error;

    fn try_from(page: Page) -> std::result::Result<Self, Self::Error> {
        let PropertyValue::Date { id: _, date } = &page.properties.properties["Event time"] else {
            return Err(anyhow::anyhow!("No event time"));
        };
        let Some(date) = date else {
            return Err(anyhow::anyhow!("No event time"));
        };

        Ok(Self {
            id: page.id.to_string(),
            title: page.title().unwrap_or("No Title".to_string()),
            changed: page.last_edited_time,
            start: date.start.clone(),
            end: date.end.clone(),
        })
    }
}

impl Event {
    pub fn to_ical(&self) -> IcalEvent {
        let event = IcalEventBuilder::tzid("UTC")
            .uid(self.id.clone())
            .changed_utc(fmt_datetime(&DateOrDateTime::DateTime(self.changed)));

        let start = fmt_datetime(&self.start);
        let end = match &self.end {
            Some(date) => fmt_datetime(date),
            None => String::new(),
        };
        let event = match self.start {
            DateOrDateTime::Date(_) => {
                if self.end.is_some() {
                    event.start_day(start).end_day(end)
                } else {
                    event.one_day(start)
                }
            }
            DateOrDateTime::DateTime(_) => event.start(start).end(end),
        };

        event
            .set(ical_property!("SUMMARY", &self.title.clone()))
            .set(ical_property!("DESCRIPTION", self.link_to_notion_event()))
            .build()
    }

    pub fn to_org(&self) -> String {
        let timestamp = fmt_datetime_org_format(&self.start, self.end.as_ref());
        [
            format!("* {}", self.title),
            format!("  :PROPERTIES:\n  :ID: {}\n  :END:", self.id),
            format!("  {timestamp}"),
            format!("  {}", self.link_to_notion_event()),
        ]
        .join("\n")
    }

    fn link_to_notion_event(&self) -> String {
        let title = self.title.clone();
        let title = title.replace(' ', "-");

        let id = self.id.clone();
        let id = id.replace('-', "");

        format!("https://www.notion.so/{}-{}", title, id)
    }
}

fn fmt_datetime(dt: &DateOrDateTime) -> String {
    match dt {
        DateOrDateTime::Date(date) => date.format("%Y%m%d").to_string(),
        DateOrDateTime::DateTime(dt) => dt.format("%Y%m%dT%H%M%SZ").to_string(),
    }
}

/// Return the event date as a String in org-mode format.
/// If the start and end dates are at the same day and both have a time,
/// we use SCHEDULED, otherwise we use a range.
fn fmt_datetime_org_format(start: &DateOrDateTime, end: Option<&DateOrDateTime>) -> String {
    if let Some(end) = end {
        match end {
            DateOrDateTime::Date(end_date) => match start {
                DateOrDateTime::Date(start_date) => {
                    if start_date == end_date {
                        start_date.format("<%Y-%m-%d %a>").to_string()
                    } else {
                        let mut d = start_date.format("<%Y-%m-%d %a>--").to_string();
                        d.push_str(&end_date.format("<%Y-%m-%d %a>").to_string());
                        d
                    }
                }
                DateOrDateTime::DateTime(start_dt) => {
                    if &start_dt.date_naive() == end_date {
                        start_dt.format("<%Y-%m-%d %a %H:%M>").to_string()
                    } else {
                        let mut d = start_dt.format("<%Y-%m-%d %a %H:%M>--").to_string();
                        d.push_str(&end_date.format("<%Y-%m-%d %a>").to_string());
                        d
                    }
                }
            },
            DateOrDateTime::DateTime(end_dt) => match start {
                DateOrDateTime::Date(start_date) => {
                    if start_date == &end_dt.date_naive() {
                        start_date.format("<%Y-%m-%d %a>").to_string()
                    } else {
                        let mut d = start_date.format("<%Y-%m-%d %a>--").to_string();
                        d.push_str(&end_dt.format("<%Y-%m-%d %a %H:%M>").to_string());
                        d
                    }
                }
                DateOrDateTime::DateTime(start_dt) => {
                    if &start_dt.date_naive() == &end_dt.date_naive() {
                        let mut d = start_dt.format("<%Y-%m-%d %a %H:%M").to_string();
                        d.push_str(&end_dt.format("-%H:%M>").to_string());
                        d
                    } else {
                        let mut d = start_dt.format("<%Y-%m-%d %a %H:%M>--").to_string();
                        d.push_str(&end_dt.format("<%Y-%m-%d %a %H:%M>").to_string());
                        d
                    }
                }
            },
        }
    } else {
        match start {
            DateOrDateTime::Date(date) => date.format("<%Y-%m-%d %a>").to_string(),
            DateOrDateTime::DateTime(dt) => dt.format("<%Y-%m-%d %a %H:%M>").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::event::fmt_datetime;
    use notion::models::properties::DateOrDateTime;

    #[test]
    fn format_datetime() {
        use chrono::prelude::*;

        let date_time: DateTime<Utc> = Utc.with_ymd_and_hms(2023, 11, 15, 11, 00, 00).unwrap();
        let formatted = fmt_datetime(&DateOrDateTime::DateTime(date_time));
        assert_eq!(formatted, "20231115T110000Z");
    }
}
