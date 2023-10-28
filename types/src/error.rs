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
