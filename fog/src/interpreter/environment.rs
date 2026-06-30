use std::collections::HashMap;

use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::kind::Kind;
use crate::interpreter::r#type::Type;
use crate::interpreter::r#type::kind_of;
use crate::interpreter::value::Value;
use crate::interpreter::value::value_type_of;
use crate::interpreter::variable::TypeVariable;
use crate::interpreter::variable::ValueVariable;
use crate::runtime_error;
use crate::type_check_error;

#[derive(Clone)]
pub struct Environment<'a> {
    pub variables: HashMap<String, ValueVariable>,
    pub types: HashMap<String, TypeVariable>,
    pub parent: Option<&'a Environment<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new(parent: Option<&'a Environment<'a>>) -> Self {
        Environment {
            variables: HashMap::new(),
            types: HashMap::new(),
            parent,
        }
    }

    pub fn flatten(&self) -> Environment<'static> {
        let mut variables = HashMap::new();
        let mut types = HashMap::new();

        if let Some(parent) = self.parent {
            let flat = parent.flatten();
            variables.extend(flat.variables);
            types.extend(flat.types);
        }

        variables.extend(self.variables.clone());
        types.extend(self.types.clone());

        Environment {
            variables,
            types,
            parent: None,
        }
    }

    // --- getters ---

    pub fn get_value_var(&self, name: &str, span: &Span) -> FogResult<ValueVariable> {
        if let Some(var) = self.variables.get(name) {
            return Ok(var.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.get_value_var(name, span);
        }

        Err(runtime_error!(
            Some(span.clone()),
            "variable `{}` not found in the current scope",
            name
        ))
    }

    pub fn get_type_var(&self, name: &str, span: &Span) -> FogResult<TypeVariable> {
        if let Some(var) = self.types.get(name) {
            return Ok(var.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.get_type_var(name, span);
        }

        Err(runtime_error!(
            Some(span.clone()),
            "type `{}` not found in the current scope",
            name
        ))
    }

    pub fn get_type(&self, name: &str, span: &Span) -> FogResult<Type> {
        self.get_type_var(name, span)?.get_type()
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

    // --- setters ---
    // -- annotate

    pub fn annotate_type(&mut self, name: &str, r#type: Type, span: &Span) -> FogResult<()> {
        if self.variables.contains_key(name) {
            return Err(runtime_error!(
                Some(span.clone()),
                "variable `{}` already annotated its type in the current scope",
                name
            ));
        }

        self.variables
            .insert(name.to_string(), ValueVariable::new(name, r#type));

        Ok(())
    }

    pub fn annotate_kind(&mut self, name: &str, kind: Kind, span: &Span) -> FogResult<()> {
        if self.types.contains_key(name) {
            return Err(runtime_error!(
                Some(span.clone()),
                "type `{}` already annotated its kind in the scope",
                name
            ));
        }

        self.types.insert(
            name.to_string(),
            TypeVariable {
                name: name.to_string(),
                r#type: None,
                kind,
            },
        );

        Ok(())
    }

    // -- declare

    pub fn declare_value(&mut self, name: &str, value: Value, span: &Span) -> FogResult<()> {
        if name == "_" {
            return Ok(());
        }

        let var = self
            .variables
            .get(name)
            .ok_or_else(|| runtime_error!(Some(span.clone()), "variable `{}` not found", name))?;

        if var.value.borrow().is_some() {
            return Err(runtime_error!(
                Some(span.clone()),
                "variable `{}` already declared in the current scope",
                name
            ));
        }

        let type_of_value = value_type_of(&value);
        let type_of_var = var.r#type.clone();

        if type_of_value != type_of_var {
            return Err(type_check_error!(
                Some(span.clone()),
                "type mismatch when assigning variable `{name}` with `{value}`\n\
                 expected `{type_of_var}`, found `{type_of_value}`"
            ));
        }

        *var.value.borrow_mut() = Some(value);

        Ok(())
    }

    pub fn declare_type(&mut self, name: &str, r#type: Type, span: &Span) -> FogResult<()> {
        let kind_of_declared_type = {
            let r#type = self.get_type_var(name, span)?;

            if r#type.r#type.is_some() {
                return Err(runtime_error!(
                    Some(span.clone()),
                    "type `{}` already declared",
                    name
                ));
            }

            r#type.kind.clone()
        };

        let kind_of_type = kind_of(&r#type);

        if kind_of_type != kind_of_declared_type {
            return Err(runtime_error!(
                Some(span.clone()),
                "kind mismatch when assigning to type `{}`\n\
                 expected `{}`, found `{}`",
                name,
                kind_of_declared_type.to_string(),
                kind_of_type.to_string()
            ));
        }

        let var = self.types.get_mut(name).unwrap_or_else(|| unreachable!());

        var.r#type = Some(r#type);

        Ok(())
    }
}
