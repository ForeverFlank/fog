use crate::error::FogResult;
use crate::interpreter::kind::Kind;
use crate::interpreter::r#type::Type;
use crate::interpreter::value::Value;
use crate::runtime_error;

#[derive(Clone)]
pub struct ValueVariable {
    pub name: String,
    pub value: Option<Value>,
    pub r#type: Type,
}

impl ValueVariable {
    pub fn get_value(&self) -> FogResult<Value> {
        self.value
            .clone()
            .ok_or_else(|| runtime_error!(None, "unassigned variable `{}`", self.name))
    }
}

#[derive(Clone)]
pub struct TypeVariable {
    pub name: String,
    pub r#type: Option<Type>,
    pub kind: Kind,
}

impl TypeVariable {
    pub fn get_type(&self) -> FogResult<Type> {
        self.r#type
            .clone()
            .ok_or_else(|| runtime_error!(None, "unassigned type `{}`", self.name))
    }
}
