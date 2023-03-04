use uuid::Uuid;
use sqlx::types::chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};

mod my_uuid_format {
		use uuid::Uuid;
    use serde::{self, Deserialize, Serializer, Deserializer};

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(
        uuid: &Uuid,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&uuid.to_string())
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
				Uuid::parse_str(&s).map_err(serde::de::Error::custom)
    }
}

mod my_date_format {
		use sqlx::types::chrono::NaiveDateTime;
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(
        date: &NaiveDateTime,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}


mod my_option_date_format {
		use sqlx::types::chrono::NaiveDateTime;
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(
        date: &Option<NaiveDateTime>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
				if let Some(dt) = date {
        	let s = format!("{}", dt.format(FORMAT));
        	serializer.serialize_str(&s)
				} else {
        	serializer.serialize_str("null")
				}
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
				if s == "null" {
					Ok(None)
				} else {
        	match NaiveDateTime::parse_from_str(&s, FORMAT) {
						Ok(dt) => Ok(Some(dt)),
						Err(e) => Err(serde::de::Error::custom(e))
					}
				}
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Label {
		#[serde(with = "my_uuid_format")] 
    pub id: Uuid,
    pub name: String,
		pub scope: Option<String>
}
impl Label {
	pub fn description(&self) -> String {
		if let Some(s) = &self.scope {
    	format!("{}::{}", s, self.name)
		} else {
    	self.name.clone()
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelNew {
    pub name: String,
		pub scope: Option<String>
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Activity {
	#[serde(with = "my_uuid_format")] 
	pub id: Uuid,
	pub name: String,
	pub description: Option<String>,
	pub labels: Vec<Label>,
	#[serde(with = "my_date_format")] 
	pub start: NaiveDateTime,
	#[serde(with = "my_option_date_format")] 
	pub end: Option<NaiveDateTime>
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityNew {
	pub name: String,
	pub description: Option<String>,
	pub labels: Vec<Label>,
	#[serde(with = "my_option_date_format")] 
	pub start: Option<NaiveDateTime>
}

