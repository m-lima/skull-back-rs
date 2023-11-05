#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Error {
    pub kind: Kind,
    pub message: Option<String>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Kind {
    BadRequest,
    NotFound,
    InternalError,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadRequest => f.write_str("Bad Request"),
            Self::NotFound => f.write_str("Not Found"),
            Self::InternalError => f.write_str("Internal Error"),
        }
    }
}
