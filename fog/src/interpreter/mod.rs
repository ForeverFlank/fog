use crate::interpreter::interpreter::Interpreter;

pub mod environment;
pub mod eval;
pub mod interpreter;
pub mod r#type;
pub mod value;
pub mod variable;

pub fn interpret(program: Box<Program>) {
    Interpreter::run(program);
}
