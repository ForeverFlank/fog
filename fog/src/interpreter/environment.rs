use std::collections::HashMap;

use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::eval_kind::kind_of;
use crate::interpreter::eval_type::make_data_constructor_function;
use crate::interpreter::kind::Kind;
use crate::interpreter::r#type::Type;
use crate::interpreter::r#type::nest_function_types;
use crate::interpreter::r#type::value_type_of;
use crate::interpreter::value::Value;
use crate::interpreter::variable::TypeVariable;
use crate::interpreter::variable::ValueVariable;
use crate::runtime_error;

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
            parent: parent,
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

    pub fn get_var(&self, name: &str, span: &Span) -> FogResult<ValueVariable> {
        if let Some(var) = self.variables.get(name) {
            return Ok(var.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.get_var(name, span);
        }

        Err(runtime_error!(
            Some(span.clone()),
            "variable `{}` not found in the current scope",
            name
        ))
    }

    pub fn get_type(&self, name: &str, span: &Span) -> FogResult<TypeVariable> {
        if let Some(var) = self.types.get(name) {
            return Ok(var.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.get_type(name, span);
        }

        Err(runtime_error!(
            Some(span.clone()),
            "type `{}` not found in the current scope",
            name
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

    // --- setters ---
    // -- annotate

    pub fn annotate_type(&mut self, name: &str, r#type: Type, span: &Span) -> FogResult<()> {
        if self.variables.contains_key(name) {
            return Err(runtime_error!(
                Some(span.clone()),
                "variable `{}` already annotated its type in the scope",
                name
            ));
        }

        self.variables.insert(
            name.to_string(),
            ValueVariable {
                name: name.to_string(),
                value: None,
                r#type,
            },
        );

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
        // discard
        if name == "_" {
            return Ok(());
        }

        let type_of_var = {
            let var = self.get_var(name, span)?;

            if var.value.is_some() {
                return Err(runtime_error!(
                    Some(span.clone()),
                    "variable `{}` already declared in the current scope",
                    name
                ));
            }

            var.r#type.clone()
        };

        let type_of_value = value_type_of(&value, self, span)?;

        if type_of_value != type_of_var {
            return Err(runtime_error!(
                Some(span.clone()),
                "type mismatch when assigning to variable `{}`\n\
                 expected `{}`, found `{}`",
                name,
                type_of_var.to_string(),
                type_of_value.to_string()
            ));
        }

        let var = self
            .variables
            .get_mut(name)
            .unwrap_or_else(|| unreachable!());

        var.value = Some(value);

        Ok(())
    }

    pub fn declare_type(&mut self, name: &str, r#type: Type, span: &Span) -> FogResult<()> {
        if self.types.contains_key(name) {
            return Err(runtime_error!(
                Some(span.clone()),
                "type `{}` already declared",
                name
            ));
        }

        let kind_of_declared_type = {
            let r#type = self.get_type(name, span)?;

            if r#type.r#type.is_some() {
                return Err(runtime_error!(
                    Some(span.clone()),
                    "variable `{}` already declared in the current scope",
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

        var.r#type = Some(r#type.clone());

        if let Type::Sum(..) = r#type {
            self.register_data_constructors(&r#type, span)?;
        }

        Ok(())
    }

    pub fn register_data_constructors(
        &mut self,
        parent_sum_type: &Type,
        span: &Span,
    ) -> FogResult<()> {
        let Type::Sum(ctors) = parent_sum_type else {
            return Err(runtime_error!(
                Some(span.clone()),
                "cannot register data constructors from a non-sum type `{}`",
                parent_sum_type.to_string()
            ));
        };

        for ctor in ctors {
            let ctor_type = nest_function_types(&ctor.types, parent_sum_type.clone());

            let ctor_value = make_data_constructor_function(
                ctor.tag.clone(),
                ctor.types.clone(),
                parent_sum_type.clone(),
                Vec::new(),
            );

            self.annotate_type(&ctor.tag, ctor_type, span)?;
            self.declare_value(&ctor.tag, ctor_value, span)?;
        }

        Ok(())
    }
}
