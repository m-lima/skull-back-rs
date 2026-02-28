pub fn incoming(request: &types::Request) -> (Resource, Action) {
    match request {
        types::Request::Skull(skull) => (
            Resource::Skull,
            match skull {
                types::request::Skull::List => Action::List,
                types::request::Skull::Create(_) => Action::Create,
                types::request::Skull::Update(_) => Action::Update,
                types::request::Skull::Delete(_) => Action::Delete,
            },
        ),
        types::Request::Occurrence(occurrence) => (
            Resource::Occurrence,
            match occurrence {
                types::request::Occurrence::List => Action::List,
                types::request::Occurrence::Quick => Action::Quick,
                types::request::Occurrence::Search(_) => Action::Search,
                types::request::Occurrence::Create(_) => Action::Create,
                types::request::Occurrence::Update(_) => Action::Update,
                types::request::Occurrence::Delete(_) => Action::Delete,
            },
        ),
    }
}

pub fn outgoing(response: &types::Response) -> Outcome {
    match response {
        types::Response::Error(error) => Outcome::Error(error.kind),
        types::Response::Payload(payload) => match payload {
            types::Payload::Change(types::Change::Created) => Outcome::Created,
            types::Payload::Change(types::Change::Updated) => Outcome::Updated,
            types::Payload::Change(types::Change::Deleted) => Outcome::Deleted,
            types::Payload::Skulls(_)
            | types::Payload::Quicks(_)
            | types::Payload::Occurrences(_) => Outcome::Ok,
        },
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Resource {
    Skull,
    Occurrence,
}

impl std::fmt::Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Skull => f.write_str("skull"),
            Self::Occurrence => f.write_str("occurrence"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Action {
    List,
    Quick,
    Search,
    Create,
    Update,
    Delete,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::List => f.write_str("list"),
            Self::Quick => f.write_str("quick"),
            Self::Search => f.write_str("search"),
            Self::Create => f.write_str("create"),
            Self::Update => f.write_str("update"),
            Self::Delete => f.write_str("delete"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Outcome {
    Ok,
    Created,
    Updated,
    Deleted,
    Error(types::Kind),
}

impl std::fmt::Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => f.write_str("Ok"),
            Self::Created => f.write_str("Created"),
            Self::Updated => f.write_str("Updated"),
            Self::Deleted => f.write_str("Deleted"),
            Self::Error(kind) => match kind {
                types::Kind::BadRequest => f.write_str("Bad request"),
                types::Kind::NotFound => f.write_str("Not found"),
                types::Kind::InternalError => f.write_str("Internal error"),
            },
        }
    }
}
