use crate::interpreter::r#type::Type;
use crate::interpreter::value::Value;

#[derive(Clone)]
pub struct Variable {
    pub name: String,
    pub value: Option<Value>,
    pub r#type: Type,
}
