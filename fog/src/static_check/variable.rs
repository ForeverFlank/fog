use crate::error::FogResult;
use crate::static_check::kind::Kind;
use crate::static_check::r#type::{Monotype, Type};
use crate::static_check_error;

#[derive(Clone)]
pub struct ValueVariable {
    pub name: String,
    pub scheme: Type,
    pub declared: bool,
}

impl ValueVariable {
    pub fn new(name: &str, scheme: Type, declared: bool) -> Self {
        ValueVariable {
            name: name.to_string(),
            scheme,
            declared,
        }
    }
}

#[derive(Clone)]
pub struct TypeVariable {
    pub name: String,
    pub r#type: Option<Monotype>,
    pub kind: Kind,
}

impl TypeVariable {
    pub fn get_type(&self) -> FogResult<Monotype> {
        self.r#type
            .clone()
            .ok_or_else(|| static_check_error!(None, "unassigned type `{}`", self.name))
    }
}
