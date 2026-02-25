use crate::util;

pub trait Model {
    type MODEL: serde::Serialize + serde::de::DeserializeOwned + 'static;
    const NAME: &'static str;

    fn path() -> &'static str;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Skull;
impl Model for Skull {
    type MODEL = types::Skull;
    const NAME: &'static str = "skull";

    fn path() -> &'static str {
        util::path!("skull")
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Quick;
impl Model for Quick {
    type MODEL = types::Quick;
    const NAME: &'static str = "quick";

    fn path() -> &'static str {
        util::path!("quick")
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Occurrence;
impl Model for Occurrence {
    type MODEL = types::Occurrence;
    const NAME: &'static str = "occurrence";

    fn path() -> &'static str {
        util::path!("occurrence")
    }
}
