use crate::interpreter::interpreter::Interpreter;
use crate::parser::resolved_expr::ResolvedStatement;

pub mod environment;
pub mod eval;
pub mod interpreter;
pub mod r#type;
pub mod value;
pub mod variable;

pub fn interpret(statements: Vec<ResolvedStatement>) {
    Interpreter::run(statements);
}
