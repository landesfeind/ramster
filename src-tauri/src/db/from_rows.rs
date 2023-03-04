use crate::models::*;
use sqlx::FromRow;
use sqlx::Row;
use uuid::Uuid;

impl FromRow<'_, sqlx::sqlite::SqliteRow> for Label {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> sqlx::Result<Label> {

        Ok(Label {
            id: Uuid::parse_str(row.try_get::<String, _>("id")?.as_str()).expect("Cannot parse uuid"),
						name: row.try_get("name")?,
						scope: row.try_get("scope")?
        })
    }
}
impl FromRow<'_, sqlx::sqlite::SqliteRow> for Activity {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> sqlx::Result<Activity> {
        Ok(Activity {
            id: Uuid::parse_str(row.try_get::<String, _>("id")?.as_str()).expect("Cannot parse uuid"),
						name: row.try_get("name")?,
						description: row.try_get("description")?,
						start: row.try_get("astart")?,
						end: row.try_get("aend")?,
						labels: Vec::new()
        })
    }
}

