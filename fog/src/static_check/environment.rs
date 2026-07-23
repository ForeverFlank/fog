use std::collections::HashMap;
use std::println;

use crate::error::FogResult;
use crate::error::Span;
use crate::static_check::kind::Kind;
use crate::static_check::static_check::can_unify;
use crate::static_check::static_check::unify_type;
use crate::static_check::r#type;
use crate::static_check::r#type::Monotype;
use crate::static_check::r#type::Type;
use crate::static_check::r#type::kind_of;
use crate::static_check::variable::TypeVariable;
use crate::static_check::variable::ValueVariable;
use crate::static_check_error;

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

    // --- getters ---

    pub fn get_value_var(&self, name: &str, span: &Span) -> FogResult<ValueVariable> {
        if let Some(var) = self.variables.get(name) {
            return Ok(var.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.get_value_var(name, span);
        }

        // panic!();

        Err(static_check_error!(
            Some(*span),
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

        Err(static_check_error!(
            Some(*span),
            "type `{}` not found in the current scope",
            name
        ))
    }

    pub fn get_type(&self, name: &str, span: &Span) -> FogResult<Monotype> {
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
            return Err(static_check_error!(
                Some(*span),
                "variable `{}` already annotated its type in the current scope",
                name
            ));
        }

        self.variables
            .insert(name.to_string(), ValueVariable::new(name, r#type, false));

        Ok(())
    }

    pub fn annotate_kind(&mut self, name: &str, kind: Kind, span: &Span) -> FogResult<()> {
        if self.types.contains_key(name) {
            return Err(static_check_error!(
                Some(*span),
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

    pub fn declare_var(&mut self, name: &str, r#type: Type, span: &Span) -> FogResult<()> {
        if name == "_" {
            return Ok(());
        }

        if let Some(var) = self.variables.get_mut(name) {
            // variable has been type-annotated

            if var.declared {
                return Err(static_check_error!(
                    Some(*span),
                    "variable `{}` already declared in the current scope",
                    name
                ));
            }

            // if var.r#type != r#type {
            // HACK but why?
            if !can_unify(&r#type.monotype, &var.r#type.monotype, span) {
                return Err(static_check_error!(
                    Some(*span),
                    "type mismatch when declaring variable `{name}`\n\
                     expected `{}`, found `{}`",
                    var.r#type,
                    r#type
                ));
            }

            var.declared = true;
        } else {
            // variable hasn't been type-annotated;
            // infer type from the declaration

            self.variables
                .insert(name.to_string(), ValueVariable::new(name, r#type, true));
        }

        Ok(())
    }

    pub fn declare_type(&mut self, name: &str, r#type: Monotype, span: &Span) -> FogResult<()> {
        let kind_of_declared_type = {
            let r#type = self.get_type_var(name, span)?;

            if r#type.r#type.is_some() {
                return Err(static_check_error!(
                    Some(*span),
                    "type `{}` already declared",
                    name
                ));
            }

            r#type.kind.clone()
        };

        let kind_of_type = kind_of(&r#type.clone());

        if kind_of_type != kind_of_declared_type {
            return Err(static_check_error!(
                Some(*span),
                "kind mismatch when declaring type `{}`\n\
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
