use crate::constant;

#[derive(Debug, thiserror::Error)]
#[error("Unknown argument: {0}")]
pub struct Error(String);

impl crate::PostAction for Error {
    fn post(self) {
        eprintln!();
        help(std::io::stderr());
    }
}

impl crate::Cancelable for Error {}

pub enum Command {
    List,
    Update,
    Register(std::iter::Skip<std::env::Args>),
    Dump,
    Plot(std::iter::Skip<std::env::Args>),
}

fn help<W: std::io::Write>(mut out: W) {
    let cache = constant::path::cache()
        .and_then(|p| p.to_str())
        .unwrap_or("??");
    let host = constant::path::host()
        .and_then(|p| p.to_str())
        .unwrap_or("??");

    drop(writeln!(
        out,
        "Usage: skull [COMMAND] [args...]

A command-line manager for the skull back end

Commands:
  l list               List occurrences for the last two days
  u update             Update the cache
  r register [args...] Register new occurrences
  d dump               Dump the occurrences in CSV format
  p plot     [args...] Plot an average of the occurrences
  h help               Show this help message

Environment variables:
  {:<21}Override the secret store user
  {:<21}Override the token user
  {:<21}Override the token password (base64 encoded)
  {:<21}Override the API host URL

Paths:
  Cache                {cache}
  Host                 {host}

Examples:
  skull r bla 1 now                            Register a `bla` occurrence for now
  skull r bla 1 2023-04-30T11:11:11Z           Register a `bla` occurrence for the timestamp given
  skull r bla 1 -1h, ble 2.5 now               Register a `bla` occurrence for one hour ago and 2.5 `ble` for now
  skull p bla,ble 1d/6h ..                     Plot all `bla` and `ble` in a one day sliding window over six hours steps
  skull p bla,ble 1d/6h -1w..                  Plot `bla` and `ble` since one week ago in a one day sliding window over six hours steps
  skull p bla 1d/6h 2023-01-01T00:00:00Z..-1d  Plot `bla`s since the timestamp given until one day ago in a one day sliding window over six hours steps",
        constant::ENV_SYSTEM_USER,
        constant::ENV_USER,
        constant::ENV_PASSWORD,
        constant::ENV_HOST,
    ));
}

pub fn parse() -> Result<Command, Error> {
    let mut args = std::env::args().skip(1);

    match args.next().as_deref() {
        None | Some("r" | "register") => Ok(Command::Register(args)),
        Some("l" | "list") => Ok(Command::List),
        Some("u" | "update") => Ok(Command::Update),
        Some("d" | "dump") => Ok(Command::Dump),
        Some("p" | "plot") => Ok(Command::Plot(args)),
        Some("h" | "-h" | "help" | "--help") => {
            help(std::io::stdout());
            std::process::exit(0);
        }
        Some(arg) => Err(Error(String::from(arg))),
    }
}
