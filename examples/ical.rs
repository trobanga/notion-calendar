use std::env;

use anyhow::Context;
use notion::ids::Identifier;
use notion_calendar::{CalendarFormat, NotionCalendar};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "debug")
    }
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    let calendar = NotionCalendar::new(
        std::env::var("NOTION_API_TOKEN").context(
            "No Notion API token found in either the environment variable \
                        `NOTION_API_TOKEN` or the config file!",
        )?,
        &env::var("NOTION_DB_ID").unwrap(),
        "prod_id".to_string(),
    )?;

    let users = calendar.list_users().await?;
    tracing::debug!(?users);
    let user = &users[0];
    let user_id = user.id().value();
    tracing::info!(
        "{}",
        calendar
            .calendar_for_user(user_id, CalendarFormat::Ical)
            .await?
    );
    Ok(())
}
