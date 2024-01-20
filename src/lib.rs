/// notion-ical
///
/// Create an iCalendar for a given user from Notion DB.
use anyhow::Result;
use notion::{
    ids::{AsIdentifier, DatabaseId, UserId},
    models::{
        paging::Paging,
        search::{DatabaseQuery, FilterCondition, PropertyCondition},
        users::User,
    },
    NotionApi,
};
use std::str::FromStr;

mod calendar;
mod event;

use event::Event;

pub enum CalendarFormat {
    Ical,
    Org,
}

#[derive(Clone)]
pub struct NotionCalendar {
    db_id: DatabaseId,
    ical_prod_id: String,
    notion_api: NotionApi,
}

impl NotionCalendar {
    pub fn new(api_token: String, db_id: &str, ical_prod_id: String) -> Result<Self> {
        let notion_api = NotionApi::new(api_token)?;
        let db_id = DatabaseId::from_str(db_id)?;
        Ok(Self {
            db_id,
            ical_prod_id,
            notion_api,
        })
    }

    async fn future_events_for_user(&self, user_id: &str) -> Result<Vec<Event>> {
        let mut events = vec![];

        let mut paging: Option<Paging> = None;
        let user_id = UserId::from_str(user_id)?;
        loop {
            let pages = self
                .notion_api
                .query_database(
                    self.db_id.as_id(),
                    DatabaseQuery {
                        sorts: None,
                        filter: Some(FilterCondition::And {
                            and: vec![
                                FilterCondition::Property {
                                    property: "Attendees".to_string(),
                                    condition: PropertyCondition::People(
                                        notion::models::search::PeopleCondition::Contains(
                                            user_id.clone(),
                                        ),
                                    ),
                                },
                                FilterCondition::Property {
                                    property: "Event time".to_string(),
                                    condition: PropertyCondition::Date(
                                        notion::models::search::DateCondition::NextYear,
                                    ),
                                },
                            ],
                        }),
                        paging: paging.clone(),
                    },
                )
                .await?;

            for event in pages.results.into_iter() {
                events.push(Event::try_from(event)?);
            }

            if pages.has_more {
                paging = Some(Paging {
                    start_cursor: pages.next_cursor,
                    page_size: None,
                });
                continue;
            } else {
                break;
            }
        }

        Ok(events)
    }

    pub async fn list_users(&self) -> Result<Vec<User>> {
        let users = self.notion_api.list_users().await?;
        Ok(users.results)
    }

    pub async fn calendar_for_user<S: AsRef<str>>(
        &self,
        user: S,
        format: CalendarFormat,
    ) -> Result<String> {
        let events = self.future_events_for_user(user.as_ref()).await?;
        Ok(match format {
            CalendarFormat::Ical => calendar::generate_ical_calendar(events, &self.ical_prod_id),
            CalendarFormat::Org => calendar::generate_org_calendar(events),
        })
    }
}
