use std::fmt::Display;

#[derive(Clone, PartialEq)]
pub enum Kind {
    Type,
    Function(Box<Kind>, Box<Kind>),
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Type => write!(f, "Kind"),
            Kind::Function(param_kind, return_kind) => {
                let param_str = param_kind.to_string();
                if param_str.contains(' ') {
                    write!(f, "({param_str}) -> {return_kind}")
                } else {
                    write!(f, "{param_str} -> {return_kind}")
                }
            }
        }
    }
}
