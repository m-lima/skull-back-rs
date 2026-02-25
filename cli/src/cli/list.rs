use super::into_rgb;

fn max_skull_name_len(skulls: &[types::Skull]) -> usize {
    skulls
        .iter()
        .map(|s| s.name.chars().count())
        .max()
        .unwrap_or(0)
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => unreachable!(),
    }
}

pub fn output(skulls: Vec<types::Skull>, occurrences: &[types::Occurrence]) {
    use chrono::{Datelike, Timelike};
    use rucline::crossterm::style::Colorize;

    let day_limit =
        types::Millis::from(chrono::Utc::now().with_hour(5).unwrap().timestamp_millis());
    let consumption_limit =
        types::Millis::from(chrono::Utc::now() - chrono::Duration::hours(24 * 3 / 4));

    let skull_len = max_skull_name_len(&skulls);

    let skulls = skulls
        .into_iter()
        .map(|s| (s.id, (s.name, s.color, s.limit)))
        .collect::<std::collections::HashMap<_, _>>();

    let mut consumption = std::collections::HashMap::<types::SkullId, (f32, f32)>::new();

    occurrences
        .iter()
        .rev()
        .filter_map(|o| skulls.get(&o.skull).map(|skull| (skull, o)))
        .for_each(|((skull, color, limit), occurrence)| {
            let timestamp = chrono::DateTime::<chrono::Local>::from(occurrence.millis);
            let timestamp = format!(
                "{day:02}-{month} {hour:02}:{minute:02}",
                day = timestamp.day(),
                month = month_name(timestamp.month()),
                hour = timestamp.hour(),
                minute = timestamp.minute()
            );

            let included = if occurrence.millis >= consumption_limit {
                if let Some(limit) = *limit {
                    consumption
                        .entry(occurrence.skull)
                        .or_insert((limit, 0.0))
                        .1 += occurrence.amount;
                }
                rucline::crossterm::style::style("┃").with((200, 50, 50).into())
            } else {
                rucline::crossterm::style::style(" ")
            };

            let color = into_rgb(*color);
            let bullet = rucline::crossterm::style::style('●').with(color.into());

            let line = format!(
                "{included}{bullet} {skull:<skull_len$} {amount:<8} {timestamp}",
                skull = String::from(skull).white(),
                amount = occurrence.amount,
            );
            println!(
                "{}",
                if occurrence.millis < day_limit {
                    line.on_black()
                } else {
                    line.on_dark_grey()
                }
            );
        });

    if !consumption.is_empty() {
        println!();
        for (skull, (limit, amount)) in &consumption {
            let (skull, _, _) = skulls.get(skull).unwrap();
            let line = format!("{skull:<skull_len$} {amount}/{limit}");
            println!(
                "{}",
                if amount > limit {
                    line.red()
                } else if *amount > limit - 1. {
                    line.yellow()
                } else {
                    line.white()
                }
            );
        }
    }
}
