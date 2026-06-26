pub fn format_joined<T: ToString>(items: &[T], sep: &str) -> String {
    items
        .iter()
        .map(|x: &T| x.to_string())
        .collect::<Vec<String>>()
        .join(sep)
}
