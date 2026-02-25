use crate::{cli, request};

type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Request(#[from] request::Error),
    #[error(transparent)]
    Cli(#[from] cli::Error),
}

impl crate::PostAction for Error {
    fn post(self) {
        match self {
            Self::Request(err) => err.post(),
            Self::Cli(err) => err.post(),
        }
    }
}

impl crate::Cancelable for Error {
    fn canceled(&self) -> bool {
        match self {
            Self::Request(err) => err.canceled(),
            Self::Cli(err) => err.canceled(),
        }
    }
}

pub struct Handler {
    request: request::Request,
}

impl Handler {
    pub fn new(request: request::Request) -> Self {
        Self { request }
    }

    pub async fn list(&self) -> Result {
        use chrono::Timelike;

        let start = types::Millis::from(
            chrono::Utc::now()
                .with_hour(5)
                .unwrap()
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap()
                - chrono::Duration::days(1),
        );

        let search = types::request::occurrence::Search {
            skulls: None,
            start: Some(start),
            end: None,
            limit: None,
        };

        let (skulls, occurrences) = tokio::join!(
            self.request.get_cacheable(),
            self.request.get_occurrences(search)
        );
        let skulls = skulls?;
        let occurrences = occurrences?;

        cli::list::output(skulls, &occurrences);

        Ok(())
    }

    pub async fn update(&self) -> Result {
        self.request.update().await.map_err(Into::into)
    }

    pub async fn register<Args>(&self, args: Args) -> Result
    where
        Args: Iterator<Item = String>,
    {
        let (skulls, quicks) =
            tokio::join!(self.request.get_cacheable(), self.request.get_cacheable());
        let skulls = skulls?;
        let quicks = quicks?;

        let occurrences = cli::register::input(args, &skulls, &quicks)?;

        self.request
            .post_occurrences(occurrences)
            .await
            .map_err(Into::into)
    }

    pub async fn dump(&self) -> Result {
        let (skulls, occurrences) = tokio::join!(
            self.request.get_cacheable(),
            self.request.get_occurrences(None)
        );
        let skulls = skulls?;
        let occurrences = occurrences?;

        cli::dump::output(skulls, &occurrences);

        Ok(())
    }

    pub async fn plot<I: Iterator<Item = String>>(&self, args: I) -> Result {
        let skulls = self.request.get_cacheable().await?;
        let (search, proto) = cli::plot::input(args, &skulls)?;
        let occurrences = self.request.get_occurrences(search).await?;
        cli::plot::output(&skulls, &occurrences, proto).map_err(Into::into)
    }
}

impl crate::PostAction for Handler {
    fn post(self) {
        self.request.post();
    }
}
