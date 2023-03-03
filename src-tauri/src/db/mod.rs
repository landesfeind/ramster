use crate::error::*;
use uuid;

type DbPool = sqlx::sqlite::SqlitePool;

#[derive(sqlx::FromRow, Clone, Debug, PartialEq, Eq)]
pub struct Label {
    pub id: String,
    pub name: String,
		pub scope: Option<String>
}

impl Label {
	pub async fn create<N: AsRef<str>, S: AsRef<str>>(db: &DbPool, name: N, scope: Option<S>) -> Result<Self> {
		let id = uuid::Uuid::new_v4().to_string();

		let scope_string = if let Some(s) = scope {
			Some(s.as_ref().to_owned())
		} else {
			None
		};
			
		let r = if let Some(s) = &scope_string {
			sqlx::query("INSERT INTO labels (id, name, scope) VALUES (?, ?, ?)")
			.bind(&id)
			.bind(name.as_ref())
			.bind(s)
			.execute(db)
			.await?
		} else {
			sqlx::query("INSERT INTO labels (id, name) VALUES (?, ?)")
			.bind(&id)
			.bind(name.as_ref())
			.execute(db)
			.await?
		};
		
		if r.rows_affected() != 1 {
			return Err(Error::DatabaseError("Cannot insert new label".to_owned()));
		}

		Ok(Label {
			id: id,
			name: name.as_ref().to_owned(),
			scope: scope_string
		})
	}

	pub async fn by_name<N: AsRef<str>, S: AsRef<str>>(db: &DbPool, name: N, scope: Option<S>) -> Result<Option<Self>> {
		let res = if let Some(s) = scope {
			sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE name = ? AND scope = ?")
				.bind(name.as_ref())
				.bind(s.as_ref())
				.fetch_optional(db)
				.await?
		} else {
			sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE name = ? AND scope IS NULL")
				.bind(name.as_ref())
				.fetch_optional(db)
				.await?
		};
		Ok(res)
	}

}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				if let Some(s) = &self.scope {
        	write!(f, "{}::{}", s, self.name)
				} else {
        	write!(f, "{}", self.name)
				}
    }
}


pub async fn connect() -> Result<DbPool> {
		let db = DbPool::connect("sqlite::memory:").await?;
	
		sqlx::query("CREATE TABLE labels (id VARCHAR PRIMARY KEY NOT NULL, name VARCHAR NOT NULL, scope VARCHAR DEFAULT NULL)")
			.execute(&db)
			.await?;

		Ok(db)
}



#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test] 
	async fn create_a_new_label_with_name_only() {
		let db = connect().await.expect("Cannot create test db");
		let l1 = Label::create(&db, "foo", None::<String>).await.expect("Cannot create");
		let o = Label::by_name(&db, "foo", None::<String>).await.expect("Cannot load previously created label");
		assert_eq!(l1, o.unwrap());
	}

	#[tokio::test] 
	async fn create_a_new_label_with_name_and_scope() {
		let db = connect().await.expect("Cannot create test db");
		let _ = Label::create(&db, "foo", Some("bar")).await.expect("Cannot create");
	}

}




