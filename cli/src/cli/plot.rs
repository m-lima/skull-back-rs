use super::{Error, Result, into_millis, into_rgb};

pub fn input<Args>(
    mut args: Args,
    skulls: &[types::Skull],
) -> Result<(types::request::occurrence::Search, SlidingWindowProto)>
where
    Args: Iterator<Item = String>,
{
    let selected_skulls = args.next();
    let window = args.next();
    let range = args.next();

    let space_denier = [(
        rucline::actions::Event::from(rucline::crossterm::event::KeyCode::Char(' ')),
        rucline::actions::Action::Noop,
    )]
    .into();

    let skulls_str = match selected_skulls {
        Some(skulls_str) => skulls_str,
        None => get_skulls(skulls, &space_denier)?,
    };
    let skulls = skulls_str
        .split(',')
        .map(|s| {
            skulls
                .iter()
                .find(|skull| skull.name == s)
                .map(|s| s.id)
                .ok_or(Error::UnknownSkull(String::from(s)))
        })
        .collect::<Result<_>>()?;

    if args.next().is_some() {
        return Err(Error::TooManyArgs);
    }

    let window_str = match window {
        Some(window_str) => window_str,
        None => get_window(&skulls_str, &space_denier)?,
    };
    let (window, step) = match window_str.as_str().split_once('/') {
        Some(("", _) | (_, "")) | None => return Err(Error::InvalidSlidingWindowValue(window_str)),
        Some((window, step)) => (
            i64::try_from(super::parse_duration(window)?.as_millis())
                .expect("window should always fit in an `i64`"),
            usize::try_from(super::parse_duration(step)?.as_millis())
                .expect("step should always fit in a `usize`"),
        ),
    };

    let range = match range {
        Some(range) => range,
        None => get_range(&skulls_str, &window_str, &space_denier)?,
    };

    let (start, end) = match range.split_once("..") {
        Some(("", "")) => (None, None),
        Some((start, "")) => (Some(into_millis(start)?), None),
        Some(("", end)) => (None, Some(into_millis(end)?)),
        Some((start, end)) => (Some(into_millis(start)?), Some(into_millis(end)?)),
        None => return Err(Error::InvalidRangeValue(range)),
    };

    // Example:
    // start = 4
    // end = 8
    // window = 2
    //
    // 0 1 2 3 4 5 6 7 8 9
    //    [---|         ]
    let start = start.map(|start| (i64::from(start) - window).into());
    let search = types::request::occurrence::Search {
        skulls: Some(skulls),
        start,
        end,
        limit: None,
    };
    let proto = SlidingWindowProto { window, step };

    Ok((search, proto))
}

fn get_skulls(
    skulls: &[types::Skull],
    space_denier: &rucline::actions::KeyBindings,
) -> Result<String> {
    use rucline::{crossterm::style::Colorize, prompt::Builder};

    rucline::prompt::Prompt::from("> ".white())
        .suggester_fn(|buffer| {
            if buffer.is_empty() {
                skulls
                    .iter()
                    .map(|s| std::borrow::Cow::Borrowed(s.name.as_str()))
                    .collect()
            } else {
                match buffer.rsplit_once(',') {
                    Some((head, current)) => {
                        let already_chosen = head.split(',').collect::<Vec<_>>();
                        skulls
                            .iter()
                            .map(|s| s.name.as_str())
                            .filter(|s| !already_chosen.contains(s) && s.starts_with(current))
                            .map(|s| std::borrow::Cow::Owned(format!("{head},{s}")))
                            .collect()
                    }
                    None => skulls
                        .iter()
                        .map(|s| s.name.as_str())
                        .filter(|s| s.starts_with(buffer.as_str()))
                        .map(std::borrow::Cow::Borrowed)
                        .collect(),
                }
            }
        })
        .completer_fn(|buffer| {
            if buffer.is_empty() {
                None
            } else {
                match buffer.rsplit_once(',') {
                    Some((_, "")) => None,
                    Some((head, current)) => {
                        let already_chosen = head.split(',').collect::<Vec<_>>();
                        skulls.iter().find_map(|s| {
                            let name = s.name.as_str();
                            if already_chosen.contains(&name) {
                                None
                            } else {
                                name.strip_prefix(current)
                            }
                        })
                    }
                    None => skulls
                        .iter()
                        .find_map(|s| s.name.strip_prefix(buffer.as_str())),
                }
            }
        })
        .overrider_ref(space_denier)
        .erase_after_read(true)
        .read_line()
        .map_err(Error::Terminal)?
        .some()
        .map(|s| String::from(s.trim()))
        .filter(|input| !input.is_empty())
        .ok_or(Error::Canceled)
}

fn get_window(skulls: &str, space_denier: &rucline::actions::KeyBindings) -> Result<String> {
    use rucline::{crossterm::style::Colorize, prompt::Builder};

    rucline::prompt::Prompt::from(format!("{skulls}> ").white())
        .suggester_fn(|buffer| {
            if !buffer.is_empty() && buffer.bytes().all(|b| b.is_ascii_digit()) {
                vec![
                    format!("{buffer}w"),
                    format!("{buffer}d"),
                    format!("{buffer}h"),
                    format!("{buffer}m"),
                    format!("{buffer}s"),
                ]
            } else {
                Vec::new()
            }
        })
        .overrider_ref(space_denier)
        .erase_after_read(true)
        .read_line()
        .map_err(Error::Terminal)?
        .some()
        .map(|s| String::from(s.trim()))
        .filter(|input| !input.is_empty())
        .ok_or(Error::Canceled)
}

fn get_range(
    skulls: &str,
    window: &str,
    space_denier: &rucline::actions::KeyBindings,
) -> Result<String> {
    use rucline::{crossterm::style::Colorize, prompt::Builder};

    rucline::prompt::Prompt::from(format!("{skulls}|{window}> ").white())
        .overrider_ref(space_denier)
        .erase_after_read(true)
        .read_line()
        .map_err(Error::Terminal)?
        .some()
        .filter(|input| !input.is_empty())
        .ok_or(Error::Canceled)
}

pub fn output(
    skulls: &[types::Skull],
    occurrences: &[types::Occurrence],
    proto: SlidingWindowProto,
) -> Result<()> {
    let buckets = aggregate(occurrences, proto);
    display(skulls, buckets)
}

fn aggregate(occurrences: &[types::Occurrence], proto: SlidingWindowProto) -> Buckets {
    let (min, max, mut timed_amounts) = occurrences.iter().fold(
        (i64::MAX, i64::MIN, Buckets::new()),
        |(min, max, mut acc), curr| {
            let millis = curr.millis.into();
            let amount = curr.amount;
            acc.entry(curr.skull)
                .or_default()
                .push(TimedAmount { millis, amount });
            (min.min(millis), max.max(millis), acc)
        },
    );

    let sliding_window = proto.finalize(min, max);

    for amounts in timed_amounts.values_mut() {
        amounts.sort_unstable_by_key(|a| a.millis);
    }

    timed_amounts
        .iter()
        .map(|(&skull, occurrences)| {
            let windows = collect_sum_of_sliding_window(sliding_window.iter(), occurrences);
            (skull, windows)
        })
        .collect()
}

fn collect_sum_of_sliding_window(
    iter: impl Iterator<Item = (i64, i64)>,
    occurrences: &[TimedAmount],
) -> Vec<TimedAmount> {
    let mut cursor = 0;
    iter.map(|(start, end)| {
        let sum = sum_all_occurrences_within_window(start, end, occurrences, &mut cursor);
        TimedAmount {
            millis: end,
            amount: sum,
        }
    })
    .collect()
}

// The cursor serves as a optimization to start the search further along the slice. It will be
// updated to point to the first value that is not completely outside of `start`
fn sum_all_occurrences_within_window(
    start: i64,
    end: i64,
    occurrences: &[TimedAmount],
    cursor: &mut usize,
) -> f32 {
    occurrences
        .iter()
        .skip(*cursor)
        .skip_while(|o| {
            if o.millis < start {
                *cursor += 1;
                true
            } else {
                false
            }
        })
        .take_while(|o| o.millis <= end)
        .map(|o| o.amount)
        .sum()
}

fn display(skulls: &[types::Skull], buckets: Buckets) -> Result<()> {
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;

    let buckets = buckets
        .into_iter()
        .filter_map(|(skull, data)| {
            let skull = skulls.iter().find(|s| s.id == skull)?;
            let data = data
                .into_iter()
                .map(|TimedAmount { millis, amount }| {
                    // Allow(clippy::cast_precision_loss): This is the best we can do
                    #[allow(clippy::cast_precision_loss)]
                    let millis = millis as f64;
                    let amount = f64::from(amount);

                    min_x = min_x.min(millis);
                    max_x = max_x.max(millis);
                    min_y = min_y.min(amount);
                    max_y = max_y.max(amount);

                    (millis, amount)
                })
                .collect::<Vec<_>>();
            Some((skull, data))
        })
        .collect::<Vec<_>>();

    let datasets = buckets
        .iter()
        .map(|(skull, data)| {
            ratatui::widgets::Dataset::default()
                .name(skull.name.as_str())
                .marker(ratatui::symbols::Marker::Braille)
                .graph_type(ratatui::widgets::GraphType::Line)
                .style(ratatui::style::Style::default().fg({
                    let color = into_rgb(skull.color);
                    ratatui::style::Color::Rgb(color.0, color.1, color.2)
                }))
                .data(data.as_slice())
        })
        .collect();

    let chart = ratatui::widgets::Chart::new(datasets)
        .block(ratatui::widgets::Block::default())
        .x_axis(
            ratatui::widgets::Axis::default()
                .bounds([min_x, max_x])
                .title("Date"),
        )
        .y_axis(
            ratatui::widgets::Axis::default()
                .bounds([min_y, max_y])
                .title("Sum"),
        );

    let mut canvas = Canvas::new()?;
    loop {
        canvas
            .terminal
            .draw(|f| {
                f.render_widget(chart.clone(), f.area());
            })
            .map_err(Error::Ratatui)?;

        rucline::crossterm::event::poll(std::time::Duration::from_secs(10))
            .map_err(Error::Terminal)?;
        if let rucline::crossterm::event::Event::Key(key) =
            rucline::crossterm::event::read().map_err(Error::Terminal)?
            && let rucline::crossterm::event::KeyCode::Char('q') = key.code
        {
            break;
        }
    }

    Ok(())
}

pub struct SlidingWindowProto {
    pub window: i64,
    pub step: usize,
}

impl SlidingWindowProto {
    fn finalize(self, start: i64, end: i64) -> SlidingWindow {
        SlidingWindow {
            window: self.window,
            step: self.step,
            start,
            end,
        }
    }
}

type Buckets = std::collections::HashMap<types::SkullId, Vec<TimedAmount>>;

struct TimedAmount {
    millis: i64,
    amount: f32,
}

struct SlidingWindow {
    window: i64,
    step: usize,
    start: i64,
    end: i64,
}

impl SlidingWindow {
    // Example:
    // min = 0
    // max = 10
    // window = 4
    // step = 2
    //
    //  0 1 2 3 4 5 6 7 8 9
    // [       ]
    //     [       ]
    //         [       ]
    //             [       ]
    fn iter(&self) -> impl Iterator<Item = (i64, i64)> {
        let window = self.window;
        (self.start + window..self.end)
            .step_by(self.step)
            .map(move |i| (i - window, i))
    }
}

struct Canvas {
    terminal: ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
}

impl Canvas {
    fn new() -> Result<Self> {
        use rucline::crossterm;
        use std::io::Write;

        crossterm::terminal::enable_raw_mode().map_err(Error::Terminal)?;
        let mut stdout = std::io::stdout();
        crossterm::execute!(&mut stdout, crossterm::terminal::EnterAlternateScreen)
            .map_err(Error::Terminal)?;
        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let terminal = ratatui::Terminal::new(backend).map_err(Error::Ratatui)?;
        Ok(Self { terminal })
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        use std::io::Write;

        drop(rucline::crossterm::terminal::disable_raw_mode());
        drop(rucline::crossterm::execute!(
            self.terminal.backend_mut(),
            rucline::crossterm::terminal::LeaveAlternateScreen
        ));
    }
}
