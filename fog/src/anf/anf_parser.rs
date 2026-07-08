use std::collections::HashMap;

use crate::anf::anf::ANFExpr;
use crate::anf::anf::AtomicExpr;
use crate::error::Span;
use crate::parser::core_expr::CoreDeclPattern;
use crate::parser::core_expr::CoreExpr;
use crate::parser::core_expr::CoreMatchArm;
use crate::parser::core_expr::CoreMatchArmPattern;
use crate::parser::core_expr::CoreStatement;

pub fn parse_anf(stmts: &Vec<CoreStatement>) -> Vec<ANFExpr> {
    let mut anfs = Vec::new();
    let mut var_counter = 0;

    for stmt in stmts {
        collect_stmt_to_anf(stmt, &mut anfs, &mut var_counter);
    }

    for anf in &mut anfs {
        if let ANFExpr::Atomic(AtomicExpr::Block { anfs, .. }) = anf {
            sort_anfs(anfs);
        }
    }

    anfs
}

fn sort_anfs(anfs: &mut Vec<ANFExpr>) {
    let num_nodes = anfs.len();
    let mut adj = vec![vec![]; num_nodes];

    for anf in anfs {
        match anf {
            ANFExpr::Atomic(atomic_expr) => match atomic_expr {
                AtomicExpr::Block { anfs, .. } => {
                    sort_anfs(anfs);
                }

                AtomicExpr::Literal { literal, .. } => {}

                AtomicExpr::Identifier { name, .. } => todo!(),

                AtomicExpr::Lambda {
                    param_name, body, ..
                } => todo!(),
                AtomicExpr::Tuple { items, .. } => todo!(),

                AtomicExpr::Match {
                    scrutinee, arms, ..
                } => todo!(),
            },

            ANFExpr::Declaration(core_decl_pattern, anfexpr) => todo!(),

            ANFExpr::FunctionAppl(atomic_expr, atomic_expr1) => todo!(),
        }
    }
}

// --- scope ---

struct Scope<'a> {
    name_ids: HashMap<String, usize>,
    parent: Option<&'a Scope<'a>>,
}

impl Scope<'_> {
    fn root<'a>() -> Scope<'a> {
        Scope {
            name_ids: HashMap::new(),
            parent: None,
        }
    }

    fn child<'a>(parent: &'a Scope<'_>) -> Scope<'a> {
        Scope {
            name_ids: HashMap::new(),
            parent: Some(parent),
        }
    }

    fn register_name(&mut self, name: &str, id_counter: &mut usize) -> usize {
        let id = *id_counter;
        self.name_ids.insert(name.to_string(), id);
        *id_counter += 1;

        id
    }

    fn get_id(&self, name: &str) -> usize {
        *self.name_ids.get(name).unwrap()
    }
}

// --- statement to ANFs ---

fn collect_stmt_to_anf(
    stmt: &CoreStatement,
    collected_anf: &mut Vec<ANFExpr>,
    var_counter: &mut i32,
) {
    match stmt {
        CoreStatement::VarDeclaration { pattern, expr, .. } => {
            let anf_expr = parse_expr_to_anf(expr, collected_anf, var_counter);
            let anf = ANFExpr::Declaration(pattern.clone(), anf_expr.into());
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
        CoreExpr::Block { .. }
        | CoreExpr::Identifier { .. }
        | CoreExpr::Literal { .. }
        | CoreExpr::Lambda { .. }
        | CoreExpr::Tuple { .. }
        | CoreExpr::Match { .. } => {
            ANFExpr::Atomic(parse_expr_to_atomic(expr, collected_anf, var_counter))
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
        CoreExpr::Identifier { name, span } => AtomicExpr::Identifier {
            name: name.clone(),
            span: *span,
        },

        CoreExpr::Literal { literal, span } => AtomicExpr::Literal {
            literal: literal.clone(),
            span: *span,
        },

        CoreExpr::Lambda {
            param_name,
            body,
            span,
            ..
        } => {
            let mut body_collected_anf = Vec::new();
            let body_anf = parse_expr_to_anf(body, &mut body_collected_anf, var_counter);
            let body_anf = wrap_scoped_anf(body_collected_anf, body_anf, *span);

            AtomicExpr::Lambda {
                param_name: param_name.clone(),
                body: body_anf.into(),
                span: *span,
            }
        }

        CoreExpr::Tuple { items, span } => AtomicExpr::Tuple {
            items: items
                .into_iter()
                .map(|item| parse_expr_to_anf(item, collected_anf, var_counter))
                .collect(),
            span: *span,
        },

        CoreExpr::Match {
            scrutinee,
            arms,
            span,
        } => AtomicExpr::Match {
            scrutinee: parse_expr_to_atomic(scrutinee, collected_anf, var_counter).into(),
            arms: arms
                .iter()
                .map(|arm| parse_match_arm(arm, var_counter, span))
                .collect::<Vec<(_, _)>>(),
            span: *span,
        },

        CoreExpr::Block { statements, span } => {
            let mut block_collected_anf = Vec::new();

            let (last, stmts) = statements.split_last().unwrap();

            for stmt in stmts.iter() {
                collect_stmt_to_anf(stmt, &mut block_collected_anf, var_counter);
            }

            let CoreStatement::Expression {
                expr: last_expr, ..
            } = last
            else {
                unreachable!()
            };

            let last_anf = parse_expr_to_anf(last_expr, &mut block_collected_anf, var_counter);
            block_collected_anf.push(last_anf);

            AtomicExpr::Block {
                anfs: block_collected_anf,
                span: *span,
            }
        }

        CoreExpr::FunctionAppl { callee, arg, span } => {
            let callee = parse_expr_to_atomic(callee, collected_anf, var_counter);
            let arg = parse_expr_to_atomic(arg, collected_anf, var_counter);
            let anf = ANFExpr::FunctionAppl(callee, arg);

            // generate a new temporary variable name
            let let_name = format!("$t{var_counter}");
            *var_counter += 1;

            // declare it
            let let_decl_pattern = CoreDeclPattern::Identifier {
                name: let_name.clone(),
                span: *span,
            };
            let let_expr = ANFExpr::Declaration(let_decl_pattern, anf.into());
            collected_anf.push(let_expr.clone());

            // return it
            AtomicExpr::Identifier {
                name: let_name,
                span: *span,
            }
        }
    }
}

fn parse_match_arm(
    arm: &CoreMatchArm,
    var_counter: &mut i32,
    span: &Span,
) -> (CoreMatchArmPattern, ANFExpr) {
    let pattern = arm.pattern.clone();

    let mut arm_collected_anf = Vec::new();
    let arm_anf = parse_expr_to_anf(&arm.value_expr, &mut arm_collected_anf, var_counter);
    let arm_anf = wrap_scoped_anf(arm_collected_anf, arm_anf, *span);

    (pattern, arm_anf)
}

fn wrap_scoped_anf(mut collected_anf: Vec<ANFExpr>, tail: ANFExpr, span: Span) -> ANFExpr {
    if collected_anf.is_empty() {
        tail
    } else {
        collected_anf.push(tail);

        ANFExpr::Atomic(AtomicExpr::Block {
            anfs: collected_anf,
            span,
        })
    }
}
