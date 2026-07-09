use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::anf::anf::ANFDeclPattern;
use crate::anf::anf::ANFExpr;
use crate::anf::anf::ANFValueExpr;
use crate::anf::anf::ANFVar;
use crate::anf::anf::AtomicExpr;
use crate::anf::scc;
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

    collect_stmts_to_anf(stmts, &mut top_scope, &mut anfs);

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

    fn next_id(&self) -> usize {
        let id = self.counter.get();
        self.counter.set(id + 1);
        id
    }

    fn register_name(&mut self, name: &str) -> usize {
        let id = self.next_id();
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

    fn new_temp(&self) -> ANFVar {
        let id = self.next_id();
        ANFVar {
            id: Some(id),
            name: format!("$t{id}"),
        }
    }
}

// --- statement to ANFs ---

fn collect_stmts_to_anf(
    stmts: &Vec<CoreStatement>,
    scope: &mut Scope,
    collected_anf: &mut Vec<ANFExpr>,
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
        collect_stmt_to_anf(stmt, scope, collected_anf);
    }

    // map declare LHS id(s) to their ANF expression
    let mut decl_stmt: HashMap<usize, usize> = HashMap::new();

    for (index, anf) in collected_anf.iter().enumerate() {
        if let ANFExpr::Declaration(pattern, _) = anf {
            for id in pattern.all_ids() {
                decl_stmt.insert(id, index);
            }
        }
    }

    // build the dependency graph
    let mut dep_graph: HashMap<usize, Vec<usize>> =
        decl_stmt.keys().map(|&id| (id, Vec::new())).collect();

    for anf in collected_anf.iter() {
        if let ANFExpr::Declaration(pattern, expr) = anf {
            let used_ids: Vec<usize> = expr
                .all_ids()
                .into_iter()
                .filter(|id| decl_stmt.contains_key(id))
                .collect();

            for id in pattern.all_ids() {
                dep_graph
                    .get_mut(&id)
                    .unwrap()
                    .extend(used_ids.iter().copied());
            }
        }
    }

    // solve for SCCs of the dependency graph
    let sccs = scc::tarjan_scc(dep_graph);

    // reorder the statements according to the SCCs output
    let mut old_stmts: Vec<Option<ANFExpr>> = std::mem::take(collected_anf)
        .into_iter()
        .map(Some)
        .collect();
    let mut reordered = Vec::new();

    for scc in sccs {
        for var in scc {
            let stmt = decl_stmt[&var];

            if let Some(anf) = old_stmts[stmt].take() {
                reordered.push(anf);
            }
        }
    }

    reordered.extend(old_stmts.into_iter().flatten());

    *collected_anf = reordered;
}

fn collect_stmt_to_anf(stmt: &CoreStatement, scope: &mut Scope, collected_anf: &mut Vec<ANFExpr>) {
    match stmt {
        CoreStatement::VarDeclaration { pattern, expr, .. } => {
            let anf_expr = parse_expr_to_anf(expr, scope, collected_anf);
            let anf_pattern = decl_pattern_to_anf(pattern, scope);
            let anf = ANFExpr::Declaration(anf_pattern, anf_expr.into());

            collected_anf.push(anf);
        }

        CoreStatement::Expression { expr, .. } => {
            parse_expr_to_anf(expr, scope, collected_anf);
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
) -> ANFValueExpr {
    match expr {
        CoreExpr::Block { .. }
        | CoreExpr::Identifier { .. }
        | CoreExpr::Literal { .. }
        | CoreExpr::Lambda { .. }
        | CoreExpr::Tuple { .. }
        | CoreExpr::Match { .. } => {
            ANFValueExpr::Atomic(parse_expr_to_atomic(expr, scope, collected_anf))
        }

        CoreExpr::FunctionAppl { callee, arg, .. } => {
            let callee = parse_expr_to_atomic(callee, scope, collected_anf);
            let arg = parse_expr_to_atomic(arg, scope, collected_anf);

            ANFValueExpr::FunctionAppl(callee, arg)
        }
    }
}

fn parse_expr_to_atomic(
    expr: &CoreExpr,
    scope: &mut Scope,
    collected_anf: &mut Vec<ANFExpr>,
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

            let body_anf = parse_expr_to_anf(body, &mut body_scope, &mut body_collected_anf);
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
                .map(|item| parse_expr_to_anf(item, scope, collected_anf))
                .collect(),
            span: *span,
        },

        CoreExpr::Match {
            scrutinee,
            arms,
            span,
        } => AtomicExpr::Match {
            scrutinee: parse_expr_to_atomic(scrutinee, scope, collected_anf).into(),
            arms: arms
                .iter()
                .map(|arm| parse_match_arm(arm, scope, span))
                .collect::<Vec<(_, _)>>(),
            span: *span,
        },

        CoreExpr::Block { statements, span } => {
            let mut block_collected_anf = Vec::new();

            let (last, stmts) = statements.split_last().unwrap();

            for stmt in stmts.iter() {
                collect_stmt_to_anf(stmt, scope, &mut block_collected_anf);
            }

            let CoreStatement::Expression {
                expr: last_expr, ..
            } = last
            else {
                unreachable!()
            };

            let last_anf = parse_expr_to_anf(last_expr, scope, &mut block_collected_anf);
            block_collected_anf.push(last_anf.to_anf_expr());

            AtomicExpr::Block {
                anfs: block_collected_anf,
                span: *span,
            }
        }

        CoreExpr::FunctionAppl { callee, arg, span } => {
            let callee = parse_expr_to_atomic(callee, scope, collected_anf);
            let arg = parse_expr_to_atomic(arg, scope, collected_anf);
            let anf = ANFExpr::FunctionAppl(callee, arg);

            let var = scope.new_temp();
            let let_decl_pattern = ANFDeclPattern::Single(var.clone());
            collected_anf.push(ANFExpr::Declaration(let_decl_pattern, anf.into()));

            AtomicExpr::Var { var, span: *span }
        }
    }
}

fn parse_match_arm(
    arm: &CoreMatchArm,
    scope: &mut Scope,
    span: &Span,
) -> (CoreMatchArmPattern, ANFValueExpr) {
    let pattern = arm.pattern.clone();

    let mut arm_collected_anf = Vec::new();
    let arm_anf = parse_expr_to_anf(&arm.value_expr, scope, &mut arm_collected_anf);
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

fn wrap_scoped_anf(
    mut collected_anf: Vec<ANFExpr>,
    tail: ANFValueExpr,
    span: Span,
) -> ANFValueExpr {
    if collected_anf.is_empty() {
        tail
    } else {
        collected_anf.push(tail.to_anf_expr());

        ANFValueExpr::Atomic(AtomicExpr::Block {
            anfs: collected_anf,
            span,
        })
    }
}
