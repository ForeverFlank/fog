use std::fmt;
use std::fmt::Display;

pub fn format_joined<T: ToString>(items: &Vec<T>, sep: &str) -> String {
    items
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(sep)
}

pub fn fmt_parenthesized<T: Display>(f: &mut fmt::Formatter<'_>, expr: &T) -> fmt::Result {
    let s = expr.to_string();
    if s.contains(' ') {
        write!(f, "({s})")
    } else {
        write!(f, "{s}")
    }
}
