pub enum Root {
    Forbidden,
    NotFound,
    MethodNotAllowed,
    Upgrade,
}

pub enum Rest {
    ContentLenghtMissing,
    ContentTypeMissing,
    PayloadTooLarge,
}

pub enum Payload {
    Serde,
}
