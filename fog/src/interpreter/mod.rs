use crate::anf::anf::ANFStatement;
use crate::anf::anf_parser::ANFMetaData;
use crate::error::FogResult;

pub mod environment;
pub mod eval_value;
pub mod interpreter;
pub mod value;
pub mod variable;

pub fn interpret(anfs: &Vec<ANFStatement>, anf_metadata: &ANFMetaData) -> FogResult<()> {
    interpreter::interpret(anfs, anf_metadata)
}
