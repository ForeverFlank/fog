use std::cell::RefCell;
use std::rc::Rc;

use crate::error::FogResult;
use crate::interpreter::value::Value;
use crate::runtime_error;

#[derive(Clone)]
pub struct ValueVariable {
    pub name: String,
    pub value: Rc<RefCell<Option<Value>>>,
}

impl ValueVariable {
    pub fn without_value(name: &str) -> Self {
        ValueVariable {
            name: name.to_string(),
            value: Rc::new(RefCell::new(None)),
        }
    }

    pub fn with_value(name: &str, value: Value) -> Self {
        ValueVariable {
            name: name.to_string(),
            value: Rc::new(RefCell::new(Some(value))),
        }
    }
}
