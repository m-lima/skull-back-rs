#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Request {
    Skull(Skull),
    Quick(Quick),
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
pub enum Quick {
    List,
    Create(quick::Create),
    Update(quick::Update),
    Delete(quick::Delete),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Occurrence {
    List,
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

pub mod quick {
    use super::Setter;
    use crate::{QuickId, SkullId};

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct Create {
        pub skull: SkullId,
        pub amount: f32,
    }

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct Update {
        pub id: QuickId,
        pub skull: Option<Setter<SkullId>>,
        pub amount: Option<Setter<f32>>,
    }

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct Delete {
        pub id: QuickId,
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
