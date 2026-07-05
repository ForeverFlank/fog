use crate::anf::anf::ANFExpr;
use crate::anf::anf::AtomicExpr;
use crate::parser::core_expr::CoreExpr;
use crate::parser::core_expr::CoreStatement;

pub fn parse_anf(stmts: &Vec<CoreStatement>) -> Vec<ANFExpr> {
    let mut res = Vec::new();

    for stmt in stmts {
        collect_stmt_to_anf(stmt, &mut res);
    }

    res
}

fn collect_stmt_to_anf(stmt: &CoreStatement, collected_anf: &mut Vec<ANFExpr>) {
    match stmt {
        CoreStatement::Declaration { pattern, expr, .. } => {
            let expr = collect_expr_to_anf(expr, collected_anf);
            let anf = ANFExpr::Let(pattern.clone(), expr.into());
            collected_anf.push(anf);
        }

        CoreStatement::Expression { expr, .. } => {
            collect_expr_to_anf(expr, collected_anf);
        }

        CoreStatement::TypeAnnotation { .. } => {}
    };
}

fn collect_expr_to_anf(expr: &CoreExpr, collected_anf: &mut Vec<ANFExpr>) -> ANFExpr {
    match expr {
        CoreExpr::Block { statements, .. } => {
            let (last, stmts) = statements.split_last().unwrap();

            for stmt in stmts.iter() {
                collect_stmt_to_anf(stmt, collected_anf);
            }

            let CoreStatement::Expression {
                expr: last_expr, ..
            } = last
            else {
                unreachable!()
            };

            collect_expr_to_anf(last_expr, collected_anf)
        }

        CoreExpr::Identifier { .. }
        | CoreExpr::Literal { .. }
        | CoreExpr::Lambda { .. }
        | CoreExpr::Tuple { .. }
        | CoreExpr::Match { .. } => {
            let anf = ANFExpr::Atomic(parse_expr_to_atomic(expr, collected_anf));
            collected_anf.push(anf.clone());

            anf
        }

        CoreExpr::FunctionAppl { callee, arg, .. } => {
            let callee = parse_expr_to_atomic(callee, collected_anf);
            let arg = parse_expr_to_atomic(arg, collected_anf);
            let anf = ANFExpr::FunctionAppl(callee, arg);
            collected_anf.push(anf.clone());

            anf
        }
    }
}

fn parse_expr_to_atomic(expr: &CoreExpr, collected_anf: &mut Vec<ANFExpr>) -> AtomicExpr {
    match expr {
        CoreExpr::Identifier { name, .. } => AtomicExpr::Identifier { name: name.clone() },

        CoreExpr::Literal { literal, .. } => AtomicExpr::Literal {
            literal: literal.clone(),
        },

        CoreExpr::Lambda {
            param_name, body, ..
        } => AtomicExpr::Lambda {
            param_name: param_name.clone(),
            body: collect_expr_to_anf(body, collected_anf).into(),
        },

        CoreExpr::Tuple { items, .. } => AtomicExpr::Tuple {
            items: items
                .into_iter()
                .map(|item| collect_expr_to_anf(item, collected_anf))
                .collect(),
        },

        CoreExpr::Match {
            scrutinee, arms, ..
        } => AtomicExpr::Match {
            scrutinee: parse_expr_to_atomic(scrutinee, collected_anf).into(),
            arms: arms
                .iter()
                .map(|arm| {
                    let pattern = arm.pattern.clone();
                    let value_expr = collect_expr_to_anf(&arm.value_expr, collected_anf);
                    (pattern, value_expr)
                })
                .collect::<Vec<_>>(),
        },

        CoreExpr::Block { .. } => todo!(),
        CoreExpr::FunctionAppl { .. } => todo!(),
    }
}
