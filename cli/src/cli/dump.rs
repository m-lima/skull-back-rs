pub fn output(skulls: Vec<types::Skull>, occurrences: &[types::Occurrence]) {
    use std::io::Write;

    let skull_lookup = skulls
        .into_iter()
        .map(|s| {
            let name = if s.name.contains(',') {
                format!(r#""{}""#, s.name)
            } else {
                s.name
            };
            (s.id, name)
        })
        .collect::<std::collections::HashMap<_, _>>();

    let mut stdout = std::io::stdout().lock();

    drop(writeln!(stdout, "skull,amount,millis"));
    for o in occurrences {
        let amount = o.amount;
        let millis = o.millis;
        if let Some(skull) = skull_lookup.get(&o.skull) {
            drop(writeln!(stdout, "{skull},{amount},{millis}"));
        } else {
            drop(writeln!(stdout, "<unknown:{}>,{amount},{millis}", o.skull));
        }
    }
}
