use crate::error::*;
use crate::models::*;
use uuid::Uuid;
use sqlx::Row;

pub type DbPool = sqlx::sqlite::SqlitePool;
mod from_rows;

pub async fn label_create(db: &DbPool, l: LabelNew) -> Result<Label> {
	let id = Uuid::new_v4();

	let scope_string = if let Some(s) = l.scope {
		Some(s)
	} else {
		None
	};
		
	let r = if let Some(s) = &scope_string {
		sqlx::query("INSERT INTO labels (id, name, scope) VALUES (?, ?, ?)")
		.bind(id.to_string())
		.bind(&l.name)
		.bind(s)
		.execute(db)
		.await?
	} else {
		sqlx::query("INSERT INTO labels (id, name) VALUES (?, ?)")
		.bind(id.to_string())
		.bind(&l.name)
		.execute(db)
		.await?
	};
	
	if r.rows_affected() != 1 {
		return Err(Error::DatabaseError("Cannot insert new label".to_owned()));
	}

	Ok(Label {
		id: id,
		name: l.name,
		scope: scope_string
	})
}

pub async fn label_by_id(db: &DbPool, id: Uuid) -> Result<Option<Label>> {
	Ok(
	  sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE id = ?")
	  	.bind(id.to_string())
	  	.fetch_optional(db)
	  	.await?
	)
}

pub async fn label_by_name<N: AsRef<str>, S: AsRef<str>>(db: &DbPool, name: N, scope: Option<S>) -> Result<Option<Label>> {
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

pub async fn label_search<N: AsRef<str>, S: AsRef<str>>(db: &DbPool, name: N, scope: Option<S>) -> Result<Vec<Label>> {
	let res = if let Some(s) = scope {
		sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE name LIKE ? AND scope = ?")
			.bind(format!("{}%", name.as_ref()))
			.bind(s.as_ref())
			.fetch_all(db)
			.await?
	} else {
		sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE name LIKE ? OR scope LIKE ?")
			.bind(format!("{}%", name.as_ref()))
			.bind(format!("{}%", name.as_ref()))
			.fetch_all(db)
			.await?
	};
	Ok(res)
}

pub async fn activity_start(db: &DbPool, a: ActivityNew) -> Result<Activity> {
	let id = Uuid::new_v4();

	sqlx::query("INSERT INTO activities (id, name, description, astart) VALUES (?, ?, ?, ?)")
		.bind(id.to_string())
		.bind(a.name)
		.bind(a.description)
		.bind(a.start)
		.execute(db)
		.await?;

	Ok(activity_by_id(db, id).await?.expect("Cannot load newly created activity"))
}

pub async fn activity_by_id(db: &DbPool, id: Uuid) -> Result<Option<Activity>> {
	let res = sqlx::query_as::<_, Activity>("SELECT * FROM activities WHERE id = ?")
		.bind(id.to_string())
		.fetch_optional(db)
		.await?;
	
	if let Some(mut a) = res {
		let labs = sqlx::query_as::<_, Label>("SELECT * FROM labels l, activity_labels al WHERE l.id = al.label_id AND al.activity_id = ?")
			.bind(id.to_string())
			.fetch_all(db)
			.await?;
		if labs.len() > 0 {
			a.labels = labs;
		}

		return Ok(Some(a))
	}

	Ok(None)
}



pub async fn connect() -> Result<DbPool> {
		let db = DbPool::connect("sqlite::memory:").await?;
	
		sqlx::query("CREATE TABLE labels (id VARCHAR PRIMARY KEY NOT NULL, name VARCHAR NOT NULL, scope VARCHAR DEFAULT NULL, UNIQUE (name, scope))")
			.execute(&db)
			.await?;

		sqlx::query("CREATE TABLE activities (id VARCHAR PRIMARY KEY NOT NULL, name VARCHAR NOT NULL, description VARCHAR DEFAULT NULL, astart TEXT NOT NULL, aend TEXT DEFAULT NULL)")
			.execute(&db)
			.await?;

		sqlx::query("CREATE TABLE activity_labels (activity_id TEXT NOT NULL REFERENCES activities (id) ON DELETE CASCADE, label_id TEXT NOT NULL REFERENCES labels (id) ON DELETE CASCADE, PRIMARY KEY (activity_id, label_id))")
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
		let l1 = label_create(&db, LabelNew { name: "foo".to_owned(), scope: None }).await.expect("Cannot create");
		let o = label_by_name(&db, "foo", None::<String>).await.expect("Cannot load previously created label");
		assert_eq!(l1, o.unwrap());
	}

	#[tokio::test] 
	async fn create_a_new_label_with_name_and_scope() {
		let db = connect().await.expect("Cannot create test db");
		let _ = label_create(&db, LabelNew { name: "foo".to_owned(), scope: Some("bar".to_owned()) }).await.expect("Cannot create");
	}

	#[tokio::test] 
	async fn create_a_new_label_with_name_only_and_search_for_it() {
		let db = connect().await.expect("Cannot create test db");
		let l1 = label_create(&db, LabelNew { name: "foo".to_owned(), scope: None }).await.expect("Cannot create");
		let res = label_search(&db, "foo", None::<String>).await.expect("search failed");
		assert_eq!(res.len(), 1);
		let res = label_search(&db, "fo", None::<String>).await.expect("search failed");
		assert_eq!(res.len(), 1);
		let res = label_search(&db, "bar", None::<String>).await.expect("search failed");
		assert_eq!(res.len(), 0);
	}

	#[tokio::test] 
	async fn create_two_label_with_name_and_scope_and_search_for_them() {
		let db = connect().await.expect("Cannot create test db");
		let l1 = label_create(&db, LabelNew { name: "foo".to_owned(), scope: Some("bar".to_owned()) }).await.expect("Cannot create");
		let l2 = label_create(&db, LabelNew { name: "foo".to_owned(), scope: Some("baz".to_owned()) }).await.expect("Cannot create");

		let l1vec = vec![l1.clone()];
		let l2vec = vec![l2.clone()];
		let lbvec = vec![l1, l2];

		let res = label_search(&db, "bar", None::<String>).await.expect("search failed");
		assert_eq!(res, l1vec);
		let res = label_search(&db, "ba", None::<String>).await.expect("search failed");
		assert_eq!(res, lbvec);
		let res = label_search(&db, "foo", None::<String>).await.expect("search failed");
		assert_eq!(res, lbvec);
		let res = label_search(&db, "fo", None::<String>).await.expect("search failed");
		assert_eq!(res, lbvec);
		let res = label_search(&db, "foo", Some("bar")).await.expect("search failed");
		assert_eq!(res, l1vec);
		let res = label_search(&db, "fo", Some("bar")).await.expect("search failed");
		assert_eq!(res, l1vec);
		let res = label_search(&db, "baz", Some("bar")).await.expect("search failed");
		assert_eq!(res.len(), 0);
		let res = label_search(&db, "ba", Some("bar")).await.expect("search failed");
		assert_eq!(res.len(), 0);
	}

}




