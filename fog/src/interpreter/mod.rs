use crate::error::FogResult;
use crate::parser::desugared_expr::DesugaredStatement;

pub mod environment;
pub mod eval_value;
pub mod interpreter;
pub mod value;
pub mod variable;

pub fn interpret(statements: &Vec<DesugaredStatement>) -> FogResult<()> {
    interpreter::interpret(statements)
}
