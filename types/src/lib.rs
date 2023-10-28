pub mod error;
pub mod request;
pub mod response;
pub mod ws;

pub use error::{Error, Kind};
pub use request::{Request, Setter};
pub use response::{Payload, Response};
pub use ws::{Message, Push};

pub type Id = i64;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct SkullId(Id);

impl From<SkullId> for Id {
    fn from(value: SkullId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Skull {
    pub id: SkullId,
    pub name: String,
    pub color: u32,
    pub icon: String,
    #[serde(rename = "unitPrice")]
    pub unit_price: f32,
    pub limit: Option<f32>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct QuickId(Id);

impl From<QuickId> for Id {
    fn from(value: QuickId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Quick {
    pub id: QuickId,
    pub skull: SkullId,
    pub amount: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct OccurrenceId(Id);

impl From<OccurrenceId> for Id {
    fn from(value: OccurrenceId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Occurrence {
    pub id: OccurrenceId,
    pub skull: SkullId,
    pub amount: f32,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub millis: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Filter {
    pub skulls: Vec<SkullId>,
    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    pub start: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    pub end: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<u32>,
}
