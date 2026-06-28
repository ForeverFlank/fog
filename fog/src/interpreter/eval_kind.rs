use crate::interpreter::kind::Kind;
use crate::interpreter::r#type::Type;

pub fn kind_of(r#type: &Type) -> Kind {
    match *r#type {
        // -> : Kind -> Kind -> Kind
        Type::Function(_, _) => Kind::Function(
            Kind::Type.into(),
            Kind::Function(Kind::Type.into(), Kind::Type.into()).into(),
        ),

        // otherwise, they're types
        _ => Kind::Type,
    }
}
