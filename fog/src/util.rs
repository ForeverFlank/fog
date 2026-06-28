pub fn format_joined<T: ToString>(items: &Vec<T>, sep: &str) -> String {
    items
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(sep)
}
