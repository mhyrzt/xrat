pub(super) fn print_geo_distribution<'a>(label: &str, values: impl Iterator<Item = &'a str>) {
    use std::collections::BTreeMap;

    let mut counts = BTreeMap::<String, usize>::new();
    for value in values {
        *counts.entry(value.to_string()).or_insert(0) += 1;
    }

    if counts.is_empty() {
        println!("{label}: -");
        return;
    }

    let summary = counts
        .iter()
        .map(|(value, count)| format!("{value}:{count}"))
        .collect::<Vec<_>>()
        .join(", ");
    println!("{label}: {summary}");
}
