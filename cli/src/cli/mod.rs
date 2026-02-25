pub mod dump;
pub mod list;
pub mod plot;
pub mod register;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Canceled")]
    Canceled,
    #[error("Unknown skull: {0}")]
    UnknownSkull(String),
    #[error("Failed to manipulate the terminal: {0}")]
    Terminal(rucline::Error),
    #[error("Invalid number: {0}")]
    InvalidNumber(std::num::ParseFloatError),
    #[error("Invalid amount: {0}")]
    InvalidAmount(f32),
    #[error("Invalid timestamp: {0}")]
    InvalidTime(chrono::ParseError),
    #[error("Invalid duration unit")]
    InvalidDurationUnit,
    #[error("Invalid duration value: {0}")]
    InvalidDurationValue(std::num::ParseIntError),
    #[error("Too many arguments")]
    TooManyArgs,
    #[error("Invalid range value: {0}")]
    InvalidRangeValue(String),
    #[error("Invalid sliding window value: {0}")]
    InvalidSlidingWindowValue(String),
    #[error("Failed to draw TUI: {0}")]
    Ratatui(std::io::Error),
}

impl crate::PostAction for Error {}

impl crate::Cancelable for Error {
    fn canceled(&self) -> bool {
        matches!(self, Error::Canceled)
    }
}

fn into_millis(time: impl AsRef<str>) -> Result<types::Millis> {
    let time = time.as_ref();
    if time == "now" {
        Ok(chrono::Utc::now().into())
    } else if let Some(duration) = time.strip_prefix('-') {
        parse_duration(duration)
            .map(|duration| chrono::Utc::now() - duration)
            .map(From::from)
    } else {
        chrono::DateTime::parse_from_rfc3339(time)
            .map_err(Error::InvalidTime)
            .map(From::from)
    }
}

fn parse_duration(duration: impl AsRef<str>) -> Result<std::time::Duration> {
    let duration = duration.as_ref();
    if let Some(amount) = duration.strip_suffix('w') {
        amount
            .parse::<u64>()
            .map(|amount| std::time::Duration::from_secs(amount * 3600 * 24 * 7))
            .map_err(Error::InvalidDurationValue)
    } else if let Some(amount) = duration.strip_suffix('d') {
        amount
            .parse::<u64>()
            .map(|amount| std::time::Duration::from_secs(amount * 3600 * 24))
            .map_err(Error::InvalidDurationValue)
    } else if let Some(amount) = duration.strip_suffix('h') {
        amount
            .parse::<u64>()
            .map(|amount| std::time::Duration::from_secs(amount * 3600))
            .map_err(Error::InvalidDurationValue)
    } else if let Some(amount) = duration.strip_suffix('m') {
        amount
            .parse::<u64>()
            .map(|amount| std::time::Duration::from_secs(amount * 60))
            .map_err(Error::InvalidDurationValue)
    } else if let Some(amount) = duration.strip_suffix('s') {
        amount
            .parse::<u64>()
            .map(std::time::Duration::from_secs)
            .map_err(Error::InvalidDurationValue)
    } else {
        Err(Error::InvalidDurationUnit)
    }
}

fn into_rgb(color: u32) -> (u8, u8, u8) {
    let r = ((color & 0xff_00_00) >> 16) as u8;
    let g = ((color & 0xff_00) >> 8) as u8;
    let b = (color & 0xff) as u8;
    (r, g, b)
}
