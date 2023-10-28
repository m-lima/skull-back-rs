pub trait Mode: sealed::Sealed + Sized + Send + 'static {
    type SerializeError: std::fmt::Display + Send;
    type DeserializeError: std::fmt::Display + Send;

    fn mode() -> &'static str;
    fn serialize(
        response: types::Message,
    ) -> Result<axum::extract::ws::Message, Self::SerializeError>;
    fn deserialize(bytes: Vec<u8>) -> Result<types::Request, Self::DeserializeError>;
}

impl Mode for String {
    type SerializeError = serde_json::Error;
    type DeserializeError = serde_json::Error;

    fn mode() -> &'static str {
        "text"
    }

    fn serialize(
        response: types::Message,
    ) -> Result<axum::extract::ws::Message, Self::SerializeError> {
        serde_json::to_string(&response).map(axum::extract::ws::Message::Text)
    }

    fn deserialize(bytes: Vec<u8>) -> Result<types::Request, Self::DeserializeError> {
        serde_json::from_slice(&bytes)
    }
}

impl Mode for Vec<u8> {
    type SerializeError = rmp_serde::encode::Error;
    type DeserializeError = rmp_serde::decode::Error;

    fn mode() -> &'static str {
        "binary"
    }

    fn serialize(
        response: types::Message,
    ) -> Result<axum::extract::ws::Message, Self::SerializeError> {
        rmp_serde::to_vec(&response).map(axum::extract::ws::Message::Binary)
    }

    fn deserialize(bytes: Vec<u8>) -> Result<types::Request, Self::DeserializeError> {
        rmp_serde::from_slice(&bytes)
    }
}

mod sealed {
    pub trait Sealed {}

    impl Sealed for String {}
    impl Sealed for Vec<u8> {}
}
