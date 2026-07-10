use std::collections::HashMap;

use crate::error::FogResult;
use crate::interpreter::value::Value;
use crate::interpreter::variable::ValueVariable;
use crate::runtime_error;

#[derive(Clone)]
pub struct Environment<'a> {
    pub variables: HashMap<String, ValueVariable>,
    pub parent: Option<&'a Environment<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new(parent: Option<&'a Environment<'a>>) -> Self {
        Environment {
            variables: HashMap::new(),
            parent,
        }
    }

    pub fn flatten(&self) -> Environment<'static> {
        let mut variables = HashMap::new();

        if let Some(parent) = self.parent {
            let flat = parent.flatten();
            variables.extend(flat.variables);
        }

        variables.extend(self.variables.clone());

        Environment {
            variables,
            parent: None,
        }
    }

    // --- getters ---

    pub fn get_value_var(&self, name: &str) -> FogResult<ValueVariable> {
        if let Some(var) = self.variables.get(name) {
            return Ok(var.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.get_value_var(name);
        }

        Err(runtime_error!(
            // Some(*span),
            None,
            "variable `{}` not found in the current scope",
            name
        ))
    }

    // --- setters ---

    // -- declare

    pub fn declare_value(&mut self, name: &str, value: Value) -> FogResult<()> {
        if name == "_" {
            return Ok(());
        }

        if let Some(var) = self.variables.get(name) {
            if var.value.borrow().is_some() {
                return Err(runtime_error!(
                    // Some(*span),
                    None,
                    "variable `{}` already declared in the current scope",
                    name
                ));
            }

            *var.value.borrow_mut() = Some(value);
        } else {
            self.variables
                .insert(name.to_string(), ValueVariable::new(name, value));
        }

        Ok(())
    }
}
