use crate::error::FogResult;
use crate::static_check::kind::Kind;
use crate::static_check::r#type::Type;
use crate::static_check_error;

#[derive(Clone)]
pub struct ValueVariable {
    pub name: String,
    pub r#type: Type,
    pub declared: bool,
}

impl ValueVariable {
    pub fn new(name: &str, r#type: Type, declared: bool) -> Self {
        ValueVariable {
            name: name.to_string(),
            r#type,
            declared,
        }
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
            .ok_or_else(|| static_check_error!(None, "unassigned type `{}`", self.name))
    }
}
