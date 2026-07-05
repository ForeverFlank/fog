use crate::error::FogResult;
use crate::runtime_error;
use crate::static_check::kind::Kind;
use crate::static_check::r#type::Type;

#[derive(Clone)]
pub struct VarVariable {
    pub name: String,
    pub r#type: Type,
    pub declared: bool,
}

impl VarVariable {
    pub fn new(name: &str, r#type: Type, declared: bool) -> Self {
        VarVariable {
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
            .ok_or_else(|| runtime_error!(None, "unassigned type `{}`", self.name))
    }
}
