pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid value for `{0}`")]
    InvalidParameter(&'static str),
    #[error("conflict of field `{0}` with `{1}`")]
    ConflictingField(&'static str, &'static str),
    #[error("entry not found for `{0}`")]
    NotFound(types::Id),
    #[error("no changes specified")]
    NoChanges,

    #[error(transparent)]
    Sqlx(sqlx::Error),
    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("referenced ID does not exist")]
    ForeignKey,
    #[error("entry fails constraint check: {0}")]
    Constraint(String),
    #[error("entry already exists: {0}")]
    DuplicateEntry(String),
}

impl Error {
    #[must_use]
    pub fn kind(&self) -> types::Kind {
        match self {
            Self::InvalidParameter(_)
            | Self::ConflictingField(_, _)
            | Self::NoChanges
            | Self::ForeignKey
            | Self::Constraint(_)
            | Self::DuplicateEntry(_) => types::Kind::BadRequest,

            Self::NotFound(_) => types::Kind::NotFound,

            Self::Sqlx(_) | Self::Migration(_) => types::Kind::InternalError,
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        if let sqlx::Error::Database(db_err) = &error {
            // TODO: Only pgsql supports this
            // if let Some(constraint) = db_err.constraint() {
            //     return Self::Constraint(String::from(constraint), error);
            // }

            if db_err.is_foreign_key_violation() {
                return Self::ForeignKey;
            }

            if db_err.is_check_violation() {
                return Self::Constraint(String::from(db_err.message()));
            }

            if db_err.is_unique_violation() {
                return Self::DuplicateEntry(String::from(db_err.message()));
            }

            if let Some(code) = db_err.code() {
                match code.as_ref() {
                    "787" => return Self::ForeignKey,
                    "275" | "1811" => return Self::Constraint(String::from(db_err.message())),
                    "2067" => return Self::DuplicateEntry(String::from(db_err.message())),
                    _ => {}
                }
            }
        }
        Self::Sqlx(error)
    }
}

impl From<Error> for types::Error {
    fn from(error: Error) -> Self {
        let kind = error.kind();

        let message = (kind != types::Kind::InternalError).then(|| error.to_string());

        Self { kind, message }
    }
}
