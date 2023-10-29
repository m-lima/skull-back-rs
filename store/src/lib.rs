// allow(clippy::missing_errors_doc): It is internal
#![allow(clippy::missing_errors_doc)]

mod error;
pub mod store;

pub use error::{Error, Result};
pub use store::Store;
