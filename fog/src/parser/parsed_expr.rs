use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

pub enum ParsedExpr {
    Identifier(String),

    Int32Literal(i32),
    Float32Literal(f32),

    Lambda {
        param_name: String,
        param_type: Box<ParsedExpr>,
        body: Rc<ParsedExpr>,
    },

    Tuple(Vec<ParsedExpr>),

    Collection(Vec<ParsedExpr>),
}

impl ParsedExpr {
    fn fmt_parenthesized(expr: &ParsedExpr, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = expr.to_string();

        if s.contains(' ') {
            write!(f, "({s})")
        } else {
            write!(f, "{s}")
        }
    }
}

impl Display for ParsedExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsedExpr::Identifier(name) => write!(f, "{name}"),

            ParsedExpr::Int32Literal(value) => write!(f, "{value}"),
            ParsedExpr::Float32Literal(value) => write!(f, "{value}"),

            ParsedExpr::Tuple(exprs) => {
                let contents: String = exprs
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "({contents})")
            }

            ParsedExpr::Lambda {
                param_name: param,
                body,
                ..
            } => {
                write!(f, "{param} => {body}")
            }

            ParsedExpr::Collection(names) => {
                for (i, name) in names.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    Self::fmt_parenthesized(name, f)?;
                }
                Ok(())
            }
        }
    }
}
