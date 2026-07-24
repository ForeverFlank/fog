use std::collections::HashMap;

use crate::static_check::r#type::Monotype;
use crate::static_check::r#type::Type;
use crate::static_check::variable::ValueVariable;

#[derive(Clone, Debug)]
pub enum Constraint {
    // constraint definition, i.e. { ... }
    Def(Vec<(String, Type)>),

    // an already-declared instance, e.g. Eq Int
    Instance(String, Vec<Monotype>),

    // combine constraints, e.g. Eq Int & { ... }
    And(Vec<Constraint>),
}

pub struct TypeClass {
    pub name: String,
    pub params: Vec<String>,
    pub constraint: Constraint,
}

pub struct TypeClassInstance {
    pub name: String,
    pub monotypes: Vec<Monotype>,
    pub methods: HashMap<String, ValueVariable>,
}
