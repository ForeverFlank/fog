use std::collections::HashMap;

use crate::interpreter::interpreter::InterpreterError;
use crate::interpreter::r#type::Type;
use crate::interpreter::r#type::is_value_of_type;
use crate::interpreter::value::Value;
use crate::interpreter::variable::Variable;

#[derive(Clone)]
pub struct Environment {
    pub variables: HashMap<String, Variable>,
    pub parent: Option<Box<Environment>>,
}

impl Environment {
    fn annotate_type(&mut self, name: &str, r#type: Type) -> Result<(), InterpreterError> {
        if self.variables.contains_key(name) {
            return Err(InterpreterError::from_string(format!(
                "variable `{}` already annotated its type in the scope",
                name
            )));
        }

        self.variables.insert(
            name.to_string(),
            Variable {
                name: name.to_string(),
                value: None,
                r#type: r#type,
            },
        );

        Ok(())
    }

    pub fn declare(&mut self, name: &str, value: Value) -> Result<(), InterpreterError> {
        let var: &mut Variable = self.variables.get_mut(name).ok_or_else(|| {
            InterpreterError::from_string(format!(
                "variable `{}` not found in the current scope",
                name
            ))
        })?;

        if var.value.is_some() {
            return Err(InterpreterError::from_string(format!(
                "variable `{}` already declared in the current scope",
                name
            )));
        }

        if !is_value_of_type(&value, &var.r#type) {
            return Err(InterpreterError::from_string(format!(
                "type mismatch when assigning to variable `{}`",
                name
            )));
        }

        var.value = Some(value);
        Ok(())
    }

    pub fn get_var(&self, name: &str) -> Result<Variable, InterpreterError> {
        if let Some(var) = self.variables.get(name) {
            return Ok(var.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.get_var(name);
        }

        Err(InterpreterError::from_string(format!(
            "variable `{}` not found in the current scope",
            name
        )))
    }
}
