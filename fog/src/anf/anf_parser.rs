use crate::anf::anf::{ANFExpr, AtomicExpr};
use crate::parser::core_expr::{CoreExpr, CoreStatement};

pub fn parse_anf(stmts: &Vec<CoreStatement>) -> Vec<ANFExpr> {
    let mut res = Vec::new();

    for stmt in stmts {
        match stmt {
            CoreStatement::Declaration { pattern, expr, .. } => {
                res.push(ANFExpr::Declaration(
                    pattern.clone(),
                    parse_expr_to_anf(expr.clone()).into(),
                ));
            }

            CoreStatement::Expression { expr, span } => todo!(),

            CoreStatement::TypeAnnotation { .. } => {}
        }
    }

    res
}

fn parse_expr_to_anf(expr: CoreExpr) -> ANFExpr {
    let atomic = match expr {
        CoreExpr::Block { statements, .. } => todo!(),

        CoreExpr::Identifier { name, .. } => AtomicExpr::Identifier { name },

        CoreExpr::Literal { literal, .. } => AtomicExpr::Literal { literal },

        CoreExpr::Lambda {
            param_name, body, ..
        } => AtomicExpr::Lambda {
            param_name,
            body: parse_expr_to_anf(*body).into(),
        },

        CoreExpr::Tuple { items, .. } => AtomicExpr::Tuple {
            items: items.into_iter().map(parse_expr_to_anf).collect(),
        },

        CoreExpr::FunctionAppl { callee, arg, .. } => todo!(),

        CoreExpr::Match {
            scrutinee,
            match_arms,
            ..
        } => todo!(),
    };

    ANFExpr::Atomic(atomic)
}
