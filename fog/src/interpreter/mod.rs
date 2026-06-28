use crate::error::FogResult;
use crate::parser::resolved_expr::ResolvedStatement;

pub mod environment;
pub mod eval_kind;
pub mod eval_type;
pub mod eval_value;
pub mod interpreter;
pub mod kind;
pub mod r#type;
pub mod typecheck;
pub mod value;
pub mod variable;

pub fn interpret(statements: &Vec<ResolvedStatement>) -> FogResult<()> {
    interpreter::interpret(statements)
}
