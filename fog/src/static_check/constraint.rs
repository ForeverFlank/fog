use crate::static_check::r#type::Type;

pub struct Constraint {
    pub name: String,
    pub type_annotations: Vec<(String, Type)>,
}
