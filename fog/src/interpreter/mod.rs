use crate::error::FogResult;
use crate::parser::core_expr::CoreStatement;

pub mod environment;
pub mod eval_type;
pub mod eval_value;
pub mod interpreter;
pub mod kind;
pub mod r#type;
pub mod type_check;
pub mod value;
pub mod variable;

pub fn interpret(statements: &Vec<CoreStatement>) -> FogResult<()> {
    interpreter::interpret(statements)
}
