use chrono::offset::TimeZone;
use procmacros::FromSqlRow;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub external_url: String,
    pub bind: Option<std::net::SocketAddr>,
    pub allowed_domains: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
pub struct Timestamp(pub chrono::DateTime<chrono::Utc>);

impl rusqlite::types::FromSql for Timestamp {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        if let rusqlite::types::ValueRef::Integer(timestamp) = value {
            Ok(Timestamp(chrono::Utc.timestamp_millis(timestamp)))
        } else {
            Err(rusqlite::types::FromSqlError::InvalidType)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromSqlRow)]
pub struct Webmention {
    pub id: Uuid,
    pub domain: String,
    pub source: String,
    pub target: String,
    pub date_added: Timestamp,
    pub date_updated: Timestamp,
}
