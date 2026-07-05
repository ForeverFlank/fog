use crate::anf::anf::ANFExpr;
use crate::anf::anf::AtomicExpr;
use crate::parser::core_expr::CoreDeclPattern;
use crate::parser::core_expr::CoreExpr;
use crate::parser::core_expr::CoreStatement;

pub fn parse_anf(stmts: &Vec<CoreStatement>) -> Vec<ANFExpr> {
    let mut res = Vec::new();

    for stmt in stmts {
        collect_stmt_to_anf(stmt, &mut res, &mut 0);
    }

    res
}

fn collect_stmt_to_anf(
    stmt: &CoreStatement,
    collected_anf: &mut Vec<ANFExpr>,
    var_counter: &mut i32,
) {
    match stmt {
        CoreStatement::VarDeclaration { pattern, expr, .. } => {
            let expr = parse_expr_to_anf(expr, collected_anf, var_counter);
            let anf = ANFExpr::Declaration(pattern.clone(), expr.into());
            collected_anf.push(anf);
        }

        CoreStatement::Expression { expr, .. } => {
            parse_expr_to_anf(expr, collected_anf, var_counter);
        }

        CoreStatement::KindAnnotation { .. }
        | CoreStatement::TypeDeclaration { .. }
        | CoreStatement::TypeAnnotation { .. } => {}
    };
}

fn parse_expr_to_anf(
    expr: &CoreExpr,
    collected_anf: &mut Vec<ANFExpr>,
    var_counter: &mut i32,
) -> ANFExpr {
    match expr {
        // atomic
        CoreExpr::Identifier { .. }
        | CoreExpr::Literal { .. }
        | CoreExpr::Lambda { .. }
        | CoreExpr::Tuple { .. }
        | CoreExpr::Match { .. } => {
            ANFExpr::Atomic(parse_expr_to_atomic(expr, collected_anf, var_counter))
        }

        // not atomic
        CoreExpr::Block { statements, .. } => {
            let (last, stmts) = statements.split_last().unwrap();

            for stmt in stmts.iter() {
                collect_stmt_to_anf(stmt, collected_anf, var_counter);
            }

            let CoreStatement::Expression {
                expr: last_expr, ..
            } = last
            else {
                unreachable!()
            };

            parse_expr_to_anf(last_expr, collected_anf, var_counter)
        }

        CoreExpr::FunctionAppl { callee, arg, .. } => {
            let callee = parse_expr_to_atomic(callee, collected_anf, var_counter);
            let arg = parse_expr_to_atomic(arg, collected_anf, var_counter);

            ANFExpr::FunctionAppl(callee, arg)
        }
    }
}

fn parse_expr_to_atomic(
    expr: &CoreExpr,
    collected_anf: &mut Vec<ANFExpr>,
    var_counter: &mut i32,
) -> AtomicExpr {
    match expr {
        // atomic -- trivial parse
        CoreExpr::Identifier { name, .. } => AtomicExpr::Identifier { name: name.clone() },

        CoreExpr::Literal { literal, .. } => AtomicExpr::Literal {
            literal: literal.clone(),
        },

        CoreExpr::Lambda {
            param_name, body, ..
        } => AtomicExpr::Lambda {
            param_name: param_name.clone(),
            body: parse_expr_to_anf(body, collected_anf, var_counter).into(),
        },

        CoreExpr::Tuple { items, .. } => AtomicExpr::Tuple {
            items: items
                .into_iter()
                .map(|item| parse_expr_to_anf(item, collected_anf, var_counter))
                .collect(),
        },

        CoreExpr::Match {
            scrutinee, arms, ..
        } => AtomicExpr::Match {
            scrutinee: parse_expr_to_atomic(scrutinee, collected_anf, var_counter).into(),
            arms: arms
                .iter()
                .map(|arm| {
                    let pattern = arm.pattern.clone();
                    let value_expr = parse_expr_to_anf(&arm.value_expr, collected_anf, var_counter);
                    (pattern, value_expr)
                })
                .collect::<Vec<_>>(),
        },

        // not atomic -- need ANF trickeries
        CoreExpr::Block { statements, .. } => {
            let (last, stmts) = statements.split_last().unwrap();

            for stmt in stmts.iter() {
                collect_stmt_to_anf(stmt, collected_anf, var_counter);
            }

            let CoreStatement::Expression {
                expr: last_expr, ..
            } = last
            else {
                unreachable!()
            };

            parse_expr_to_atomic(last_expr, collected_anf, var_counter)
        }

        CoreExpr::FunctionAppl { callee, arg, span } => {
            let callee = parse_expr_to_atomic(callee, collected_anf, var_counter);
            let arg = parse_expr_to_atomic(arg, collected_anf, var_counter);
            let anf = ANFExpr::FunctionAppl(callee, arg);

            let let_name = generate_var_name(var_counter);
            let let_pattern = CoreDeclPattern::Identifier {
                name: let_name.clone(),
                span: *span,
            };
            let let_expr = ANFExpr::Declaration(let_pattern, anf.into());
            collected_anf.push(let_expr.clone());

            AtomicExpr::Identifier { name: let_name }
        }
    }
}

fn generate_var_name(var_counter: &mut i32) -> String {
    let s = format!("$t{var_counter}");
    *var_counter += 1;

    s
}
