use std::cell::RefCell;
use std::rc::Rc;

use crate::error::FogResult;
use crate::interpreter::kind::Kind;
use crate::interpreter::r#type::Type;
use crate::interpreter::value::Value;
use crate::runtime_error;

#[derive(Clone)]
pub struct ValueVariable {
    pub name: String,
    pub value: Rc<RefCell<Option<Value>>>,
    pub r#type: Type,
}

impl ValueVariable {
    pub fn new(name: &str, r#type: Type) -> Self {
        ValueVariable {
            name: name.to_string(),
            value: Rc::new(RefCell::new(None)),
            r#type,
        }
    }

    pub fn with_value(name: &str, value: Value, r#type: Type) -> Self {
        ValueVariable {
            name: name.to_string(),
            value: Rc::new(RefCell::new(Some(value))),
            r#type,
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
