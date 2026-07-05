use crate::error::FogError;
use crate::parser::core_expr::CoreStatement;

pub mod environment;
pub mod eval_type;
pub mod kind;
pub mod r#type;
pub mod static_check;
pub mod variable;

pub fn static_check(stmts: &Vec<CoreStatement>) -> Vec<FogError> {
    static_check::check(stmts)
}
