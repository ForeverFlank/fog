use crate::error::FogResult;
use crate::interpreter::environment::Environment;
use crate::interpreter::r#type::Type;
use crate::interpreter::r#type::get_value_type;
use crate::interpreter::value::Value;

#[derive(Clone)]
pub struct Variable {
    pub name: String,
    pub value: Option<Value>,
    pub r#type: Type,
}

impl Variable {
    fn from_value(
        name: String,
        value: Value,
        env: &Environment,
    ) -> FogResult<Variable> {
        Ok(Variable {
            name,
            value: Some(value.clone()),
            r#type: get_value_type(&value, &env)?,
        })
    }
}
