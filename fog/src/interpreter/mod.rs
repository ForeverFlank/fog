use crate::anf::anf::ANFStatement;
use crate::error::FogResult;

pub mod environment;
pub mod eval_value;
pub mod interpreter;
pub mod value;
pub mod variable;

pub fn interpret(anfs: &Vec<ANFStatement>) -> FogResult<()> {
    interpreter::interpret(anfs)
}
