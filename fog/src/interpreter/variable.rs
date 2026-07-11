use std::cell::RefCell;
use std::rc::Rc;

use crate::interpreter::value::Value;

#[derive(Clone)]
pub struct ValueVariable {
    pub name: String,
    pub value: Rc<RefCell<Option<Value>>>,
}

impl ValueVariable {
    pub fn new(name: &str, value: Value) -> Self {
        ValueVariable {
            name: name.to_string(),
            value: Rc::new(RefCell::new(Some(value))),
        }
    }

    pub fn new_uninitialized(name: &str) -> Self {
        ValueVariable {
            name: name.to_string(),
            value: Rc::new(RefCell::new(None)),
        }
    }
}
