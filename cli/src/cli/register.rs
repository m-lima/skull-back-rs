use super::{Error, Result, into_millis};

struct Completions {
    options: Vec<String>,
    matcher: fuzzy_matcher::skim::SkimMatcherV2,
}

impl Completions {
    fn new(options: Vec<String>) -> Self {
        Self {
            options,
            matcher: fuzzy_matcher::skim::SkimMatcherV2::default(),
        }
    }
}

impl rucline::completion::Suggester for Completions {
    fn suggest_for(&self, buffer: &rucline::Buffer) -> Vec<std::borrow::Cow<'_, str>> {
        let buffer = buffer.as_str();
        let mut suggestions = self
            .options
            .iter()
            .filter_map(|opt| {
                self.matcher
                    .fuzzy(opt, buffer, false)
                    .map(|(score, _)| (score, std::borrow::Cow::Borrowed(opt.as_str())))
            })
            .collect::<Vec<_>>();
        suggestions.sort_unstable();
        suggestions.into_iter().map(|(_, opt)| opt).collect()
    }
}

impl rucline::completion::Completer for Completions {
    fn complete_for(&self, buffer: &rucline::Buffer) -> Option<std::borrow::Cow<'_, str>> {
        self.options.complete_for(buffer)
    }
}

pub fn input<Args>(
    args: Args,
    skulls: &[types::Skull],
    quicks: &[types::Quick],
) -> Result<Vec<types::request::occurrence::Item>>
where
    Args: Iterator<Item = String>,
{
    args.collect::<Vec<_>>()
        .join(" ")
        .split(',')
        .map(|params| parse_skull_params(params, skulls, quicks))
        .collect()
}

fn parse_skull_params(
    params: &str,
    skulls: &[types::Skull],
    quicks: &[types::Quick],
) -> Result<types::request::occurrence::Item> {
    let mut args = params.split(' ').filter(|b| !b.is_empty());

    let skull = args.next();
    let amount = args.next();
    let time = args.next();

    if args.next().is_some() {
        return Err(Error::TooManyArgs);
    }

    let skull = match skull {
        Some(s) => into_skull(skulls, s),
        None => get_skull(skulls).and_then(|s| into_skull(skulls, &s)),
    }?;

    let amount = match amount {
        Some(amount) => into_amount(amount),
        None => get_amount(skull, quicks).and_then(into_amount),
    }?;

    let millis = match time {
        Some(millis) => into_millis(millis),
        None => get_time(&skull.name, amount).and_then(into_millis),
    }?;

    Ok(types::request::occurrence::Item {
        skull: skull.id,
        amount,
        millis,
    })
}

fn get_skull(skulls: &[types::Skull]) -> Result<String> {
    use rucline::{crossterm::style::Colorize, prompt::Builder};

    let completions = Completions::new(skulls.iter().map(|s| s.name.clone()).collect());

    rucline::prompt::Prompt::from("> ".white())
        .completer_ref(&completions)
        .suggester_ref(&completions)
        .erase_after_read(true)
        .read_line()
        .map_err(Error::Terminal)?
        .some()
        .filter(|input| !input.is_empty())
        .ok_or(Error::Canceled)
}

fn into_skull<'a>(skulls: &'a [types::Skull], skull: &str) -> Result<&'a types::Skull> {
    skulls
        .iter()
        .find(|s| skull == s.name)
        .ok_or(Error::UnknownSkull(String::from(skull)))
}

fn get_amount(skull: &types::Skull, quicks: &[types::Quick]) -> Result<String> {
    use rucline::{crossterm::style::Colorize, prompt::Builder};

    let options = quicks
        .iter()
        .filter(|q| q.skull == skull.id)
        .map(|q| q.amount.to_string())
        .collect::<Vec<_>>();

    rucline::prompt::Prompt::from(format!("{}> ", skull.name).white())
        .buffer(rucline::Buffer::from(
            options.first().map_or("1", String::as_str),
        ))
        .completer_ref(&options)
        .suggester_ref(&options)
        .erase_after_read(true)
        .read_line()
        .map_err(Error::Terminal)?
        .some()
        .filter(|input| !input.is_empty())
        .ok_or(Error::Canceled)
}

fn into_amount<A>(amount: A) -> Result<f32>
where
    A: AsRef<str>,
{
    let amount = amount.as_ref().parse().map_err(Error::InvalidNumber)?;
    if amount > 0.0 {
        Ok(amount)
    } else {
        Err(Error::InvalidAmount(amount))
    }
}

fn get_time(skull: &str, amount: f32) -> Result<String> {
    use rucline::{crossterm::style::Colorize, prompt::Builder};

    rucline::prompt::Prompt::from(format!("{skull}|{amount}> ").white())
        .buffer(rucline::Buffer::from(
            chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now()).to_rfc3339(),
        ))
        .read_line()
        .map_err(Error::Terminal)?
        .some()
        .filter(|input| !input.is_empty())
        .ok_or(Error::Canceled)
}
