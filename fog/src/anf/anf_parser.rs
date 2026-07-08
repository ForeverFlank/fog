use std::cell::Cell;
use std::collections::HashMap;
use std::convert::identity;
use std::rc::Rc;

use crate::anf::anf::ANFDeclPattern;
use crate::anf::anf::ANFExpr;
use crate::anf::anf::ANFVar;
use crate::anf::anf::AtomicExpr;
use crate::error::Span;
use crate::parser::core_expr::CoreDeclPattern;
use crate::parser::core_expr::CoreExpr;
use crate::parser::core_expr::CoreMatchArm;
use crate::parser::core_expr::CoreMatchArmPattern;
use crate::parser::core_expr::CoreStatement;
use crate::parser::core_expr::CoreTupleDeclPattern;

pub fn parse_anf(stmts: &Vec<CoreStatement>) -> Vec<ANFExpr> {
    let mut anfs = Vec::new();

    let mut top_scope = Scope::new_root();
    let mut var_counter = 0;

    collect_stmts_to_anf(stmts, &mut top_scope, &mut anfs, &mut var_counter);

    anfs
}

// --- scope ---

struct Scope<'a> {
    names: HashMap<String, usize>,
    parent: Option<&'a Scope<'a>>,
    counter: Rc<Cell<usize>>,
}

impl<'a> Scope<'a> {
    fn new_root() -> Self {
        Self {
            names: HashMap::new(),
            parent: None,
            counter: Rc::new(Cell::new(0)),
        }
    }

    fn new_child(parent: &'a Scope<'a>) -> Self {
        Self {
            names: HashMap::new(),
            parent: Some(parent),
            counter: Rc::clone(&parent.counter),
        }
    }

    fn register_name(&mut self, name: &str) -> usize {
        let id = self.counter.get();
        self.counter.set(id + 1);
        self.names.insert(name.to_string(), id);
        id
    }

    fn get_id(&self, name: &str) -> Option<usize> {
        if let Some(&id) = self.names.get(name) {
            Some(id)
        } else if let Some(parent) = self.parent {
            parent.get_id(name)
        } else {
            None
        }
    }
}

// --- statement to ANFs ---

fn collect_stmts_to_anf(
    stmts: &Vec<CoreStatement>,
    scope: &mut Scope,
    collected_anf: &mut Vec<ANFExpr>,
    var_counter: &mut i32,
) {
    for stmt in stmts {
        match stmt {
            CoreStatement::VarDeclaration { pattern, .. } => {
                for name in pattern.all_identifiers() {
                    scope.register_name(name);
                }
            }

            CoreStatement::Expression { .. } => {}

            // types erased -- no works needed!
            CoreStatement::TypeDeclaration { .. }
            | CoreStatement::KindAnnotation { .. }
            | CoreStatement::TypeAnnotation { .. } => {}
        }
    }

    for stmt in stmts {
        collect_stmt_to_anf(stmt, scope, collected_anf, var_counter);
    }

    let mut edges = Vec::new();

    for anf in collected_anf {
        match anf {
            ANFExpr::Declaration(pattern, expr) => {
                for id in pattern.all_ids() {
                    edges.push((id, ()))
                }
            }

            ANFExpr::FunctionAppl(callee, arg) => {}

            // this one's always the last in a scope
            ANFExpr::Atomic(atomic_expr) => {}
        }
    }

    // for id in scope.name_ids().values()
}

fn collect_stmt_to_anf(
    stmt: &CoreStatement,
    scope: &mut Scope,
    collected_anf: &mut Vec<ANFExpr>,
    var_counter: &mut i32,
) {
    match stmt {
        CoreStatement::VarDeclaration { pattern, expr, .. } => {
            let anf_expr = parse_expr_to_anf(expr, scope, collected_anf, var_counter);
            let anf_pattern = decl_pattern_to_anf(pattern, scope);
            let anf = ANFExpr::Declaration(anf_pattern, anf_expr.into());

            collected_anf.push(anf);
        }

        CoreStatement::Expression { expr, .. } => {
            parse_expr_to_anf(expr, scope, collected_anf, var_counter);
        }

        CoreStatement::KindAnnotation { .. }
        | CoreStatement::TypeDeclaration { .. }
        | CoreStatement::TypeAnnotation { .. } => {}
    };
}

fn parse_expr_to_anf(
    expr: &CoreExpr,
    scope: &mut Scope,
    collected_anf: &mut Vec<ANFExpr>,
    var_counter: &mut i32,
) -> ANFExpr {
    match expr {
        CoreExpr::Block { .. }
        | CoreExpr::Identifier { .. }
        | CoreExpr::Literal { .. }
        | CoreExpr::Lambda { .. }
        | CoreExpr::Tuple { .. }
        | CoreExpr::Match { .. } => ANFExpr::Atomic(parse_expr_to_atomic(
            expr,
            scope,
            collected_anf,
            var_counter,
        )),

        CoreExpr::FunctionAppl { callee, arg, .. } => {
            let callee = parse_expr_to_atomic(callee, scope, collected_anf, var_counter);
            let arg = parse_expr_to_atomic(arg, scope, collected_anf, var_counter);

            ANFExpr::FunctionAppl(callee, arg)
        }
    }
}

fn parse_expr_to_atomic(
    expr: &CoreExpr,
    scope: &mut Scope,
    collected_anf: &mut Vec<ANFExpr>,
    var_counter: &mut i32,
) -> AtomicExpr {
    match expr {
        // atomic -- trivial parse
        CoreExpr::Identifier { name, span } => AtomicExpr::Var {
            var: ANFVar {
                id: scope.get_id(name),
                name: name.to_string(),
            },
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
            let mut body_scope = Scope::new_child(scope);
            body_scope.register_name(param_name);

            let body_anf =
                parse_expr_to_anf(body, &mut body_scope, &mut body_collected_anf, var_counter);
            let body_anf = wrap_scoped_anf(body_collected_anf, body_anf, *span);

            AtomicExpr::Lambda {
                param: ANFVar {
                    id: body_scope.get_id(param_name),
                    name: param_name.to_string(),
                },
                body: body_anf.into(),
                span: *span,
            }
        }

        CoreExpr::Tuple { items, span } => AtomicExpr::Tuple {
            items: items
                .into_iter()
                .map(|item| parse_expr_to_anf(item, scope, collected_anf, var_counter))
                .collect(),
            span: *span,
        },

        CoreExpr::Match {
            scrutinee,
            arms,
            span,
        } => AtomicExpr::Match {
            scrutinee: parse_expr_to_atomic(scrutinee, scope, collected_anf, var_counter).into(),
            arms: arms
                .iter()
                .map(|arm| parse_match_arm(arm, scope, var_counter, span))
                .collect::<Vec<(_, _)>>(),
            span: *span,
        },

        CoreExpr::Block { statements, span } => {
            let mut block_collected_anf = Vec::new();

            let (last, stmts) = statements.split_last().unwrap();

            for stmt in stmts.iter() {
                collect_stmt_to_anf(stmt, scope, &mut block_collected_anf, var_counter);
            }

            let CoreStatement::Expression {
                expr: last_expr, ..
            } = last
            else {
                unreachable!()
            };

            let last_anf =
                parse_expr_to_anf(last_expr, scope, &mut block_collected_anf, var_counter);
            block_collected_anf.push(last_anf);

            AtomicExpr::Block {
                anfs: block_collected_anf,
                span: *span,
            }
        }

        CoreExpr::FunctionAppl { callee, arg, span } => {
            let callee = parse_expr_to_atomic(callee, scope, collected_anf, var_counter);
            let arg = parse_expr_to_atomic(arg, scope, collected_anf, var_counter);
            let anf = ANFExpr::FunctionAppl(callee, arg);

            // generate a new temporary variable name
            let let_name = format!("$t{var_counter}");
            *var_counter += 1;

            scope.register_name(&let_name);

            // declare it
            let let_decl_pattern = ANFDeclPattern::Single(ANFVar {
                id: scope.get_id(&let_name),
                name: let_name.clone(),
            });
            let let_expr = ANFExpr::Declaration(let_decl_pattern, anf.into());
            collected_anf.push(let_expr.clone());

            // return it
            AtomicExpr::Var {
                var: ANFVar {
                    id: scope.get_id(&let_name),
                    name: let_name,
                },
                span: *span,
            }
        }
    }
}

fn parse_match_arm(
    arm: &CoreMatchArm,
    scope: &mut Scope,
    var_counter: &mut i32,
    span: &Span,
) -> (CoreMatchArmPattern, ANFExpr) {
    let pattern = arm.pattern.clone();

    let mut arm_collected_anf = Vec::new();
    let arm_anf = parse_expr_to_anf(&arm.value_expr, scope, &mut arm_collected_anf, var_counter);
    let arm_anf = wrap_scoped_anf(arm_collected_anf, arm_anf, *span);

    (pattern, arm_anf)
}

fn decl_pattern_to_anf(pattern: &CoreDeclPattern, scope: &Scope) -> ANFDeclPattern {
    match pattern {
        CoreDeclPattern::Identifier { name, .. } => ANFDeclPattern::Single(ANFVar {
            id: scope.get_id(name),
            name: name.clone(),
        }),

        CoreDeclPattern::Tuple { items, .. } => ANFDeclPattern::Tuple(
            items
                .iter()
                .map(|item| tuple_decl_pattern_to_anf(item, scope))
                .collect(),
        ),
    }
}

fn tuple_decl_pattern_to_anf(pattern: &CoreTupleDeclPattern, scope: &Scope) -> ANFDeclPattern {
    match pattern {
        CoreTupleDeclPattern::Identifier { name, .. } => ANFDeclPattern::Single(ANFVar {
            id: scope.get_id(name),
            name: name.clone(),
        }),

        CoreTupleDeclPattern::Tuple { items, .. } => ANFDeclPattern::Tuple(
            items
                .iter()
                .map(|item| tuple_decl_pattern_to_anf(item, scope))
                .collect(),
        ),
    }
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
