use std::collections::HashMap;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::r#type::Type;
use crate::interpreter::r#type::value_type_of;
use crate::interpreter::value::Value;
use crate::interpreter::variable::Variable;

#[derive(Clone)]
pub struct Environment {
    pub variables: HashMap<String, Variable>,
    pub types: HashMap<String, Type>,
    pub parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new(parent: Option<Box<Environment>>) -> Self {
        Environment {
            variables: HashMap::new(),
            types: HashMap::new(),
            parent: parent,
        }
    }

    pub fn annotate_type(&mut self, name: &str, r#type: Type, span: &Span) -> FogResult<()> {
        if self.variables.contains_key(name) {
            return Err(FogError::runtime(
                format!(
                    "variable `{}` already annotated its type in the scope",
                    name
                ),
                Some(span.clone()),
            ));
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

    pub fn declare_value(&mut self, name: &str, value: Value, span: &Span) -> FogResult<()> {
        // discard
        if name == "_" {
            return Ok(());
        }

        let var_type: Type = {
            let var: &mut Variable = self.variables.get_mut(name).ok_or_else(|| {
                FogError::runtime(
                    format!("variable `{}` not found in the current scope", name),
                    Some(span.clone()),
                )
            })?;

            if var.value.is_some() {
                return Err(FogError::runtime(
                    format!("variable `{}` already declared in the current scope", name),
                    Some(span.clone()),
                ));
            }

            var.r#type.clone()
        };

        let value_type: Type = value_type_of(&value, self, span)?;

        if value_type != var_type {
            return Err(FogError::runtime(
                format!(
                    "type mismatch when assigning to variable `{}`\n\
                     expected `{}`, found `{}`",
                    name,
                    var_type.to_string(),
                    value_type.to_string()
                ),
                Some(span.clone()),
            ));
        }

        let var: &mut Variable = self
            .variables
            .get_mut(name)
            .unwrap_or_else(|| unreachable!());
        var.value = Some(value);

        Ok(())
    }

    pub fn declare_type(&mut self, name: &str, r#type: Type, span: &Span) -> FogResult<()> {
        if self.types.contains_key(name) {
            return Err(FogError::runtime(
                format!("type `{}` already declared", name),
                Some(span.clone()),
            ));
        }
        self.types.insert(name.to_string(), r#type);

        Ok(())
    }

    pub fn get_var(&self, name: &str) -> FogResult<Variable> {
        if let Some(var) = self.variables.get(name) {
            return Ok(var.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.get_var(name);
        }

        Err(FogError::runtime(
            format!("variable `{}` not found in the current scope", name),
            None,
        ))
    }

    pub fn get_type(&self, name: &str) -> FogResult<Type> {
        if let Some(var) = self.types.get(name) {
            return Ok(var.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.get_type(name);
        }

        Err(FogError::runtime(
            format!("type `{}` not found in the current scope", name),
            None,
        ))
    }

    pub fn contains_type(&self, name: &str) -> bool {
        if self.types.contains_key(name) {
            return true;
        }

        if let Some(parent) = &self.parent {
            return parent.contains_type(name);
        }

        false
    }
}
