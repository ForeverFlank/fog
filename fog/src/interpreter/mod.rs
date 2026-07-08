use crate::anf::anf::ANFExpr;
use crate::error::FogResult;

pub mod environment;
pub mod eval_value;
pub mod interpreter;
pub mod value;
pub mod variable;

pub fn interpret(statements: &Vec<ANFExpr>) -> FogResult<()> {
    interpreter::interpret(statements)
}
