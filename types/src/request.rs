#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Request {
    Skull(Skull),
    Occurrence(Occurrence),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Skull {
    List,
    Create(skull::Create),
    Update(skull::Update),
    Delete(skull::Delete),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Occurrence {
    List,
    Quick,
    Search(occurrence::Search),
    Create(occurrence::Create),
    Update(occurrence::Update),
    Delete(occurrence::Delete),
}

pub mod skull {
    use super::Setter;
    use crate::SkullId;

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct Create {
        pub name: String,
        pub color: u32,
        pub icon: String,
        pub price: f32,
        pub limit: Option<f32>,
    }

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct Update {
        pub id: SkullId,
        pub name: Option<Setter<String>>,
        pub color: Option<Setter<u32>>,
        pub icon: Option<Setter<String>>,
        pub price: Option<Setter<f32>>,
        pub limit: Option<Setter<Option<f32>>>,
    }

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct Delete {
        pub id: SkullId,
    }
}

pub mod occurrence {
    use super::Setter;
    use crate::{Millis, OccurrenceId, SkullId};

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct Search {
        pub skulls: Option<std::collections::HashSet<SkullId>>,
        pub start: Option<Millis>,
        pub end: Option<Millis>,
        pub limit: Option<usize>,
    }

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct Create {
        pub items: Vec<Item>,
    }

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct Item {
        pub skull: SkullId,
        pub amount: f32,
        pub millis: Millis,
    }

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct Update {
        pub id: OccurrenceId,
        pub skull: Option<Setter<SkullId>>,
        pub amount: Option<Setter<f32>>,
        pub millis: Option<Setter<Millis>>,
    }

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct Delete {
        pub id: OccurrenceId,
    }

    #[cfg(feature = "query")]
    pub mod query {
        use super::{Millis, Search, SkullId};

        impl Search {
            #[must_use]
            pub fn to_query(&self) -> String {
                [
                    self.skulls.as_ref().map(|skulls| {
                        let skulls = skulls
                            .iter()
                            .map(|id| String::from(itoa::Buffer::new().format(i64::from(*id))))
                            .collect::<Vec<_>>()
                            .join(",");
                        format!("skulls={skulls}")
                    }),
                    self.start.map(|start| {
                        format!("start={}", itoa::Buffer::new().format(i64::from(start)))
                    }),
                    self.end
                        .map(|end| format!("end={}", itoa::Buffer::new().format(i64::from(end)))),
                    self.limit
                        .map(|limit| format!("limit={}", itoa::Buffer::new().format(limit))),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join("&")
            }

            pub fn from_query(query: &str) -> Result<Self, Error> {
                let mut search = Self {
                    skulls: None,
                    start: None,
                    end: None,
                    limit: None,
                };
                for (key, value) in query.split('&').filter_map(|p| p.split_once('=')) {
                    match key {
                        "skulls" => {
                            search.skulls = Some(
                                value
                                    .split(',')
                                    .map(|id| id.parse::<i64>().map(SkullId))
                                    .collect::<Result<_, _>>()
                                    .map_err(|error| Error {
                                        field: "skulls",
                                        error,
                                    })?,
                            );
                        }
                        "start" => {
                            search.start =
                                Some(value.parse::<i64>().map(Millis::from).map_err(|error| {
                                    Error {
                                        field: "start",
                                        error,
                                    }
                                })?);
                        }
                        "end" => {
                            search.end =
                                Some(value.parse::<i64>().map(Millis::from).map_err(|error| {
                                    Error {
                                        field: "end",
                                        error,
                                    }
                                })?);
                        }
                        "limit" => {
                            search.limit = Some(value.parse::<usize>().map_err(|error| Error {
                                field: "limit",
                                error,
                            })?);
                        }
                        _ => {}
                    }
                }
                Ok(search)
            }
        }

        #[derive(Debug, Clone, Eq, PartialEq)]
        pub struct Error {
            field: &'static str,
            error: std::num::ParseIntError,
        }

        impl std::error::Error for Error {}

        impl std::fmt::Display for Error {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Failed to parse field `{}`: {}", self.field, self.error)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Setter<T> {
    pub set: T,
}

impl<T> From<T> for Setter<T> {
    fn from(set: T) -> Self {
        Setter { set }
    }
}

impl<T> Setter<T> {
    pub fn set(self) -> T {
        self.set
    }
}
