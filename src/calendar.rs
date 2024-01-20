use ical::generator::{Emitter, IcalCalendarBuilder};

use crate::Event;

pub fn generate_ical_calendar(events: Vec<Event>, prod_id: &str) -> String {
    let mut cal = IcalCalendarBuilder::version("2.0")
        .gregorian()
        .prodid(prod_id)
        .build();

    for event in events {
        cal.events.push(event.to_ical());
    }

    cal.generate()
}

pub fn generate_org_calendar(events: Vec<Event>) -> String {
    events
        .into_iter()
        .map(|event| event.to_org())
        .collect::<Vec<String>>()
        .join("\n\n")
}
