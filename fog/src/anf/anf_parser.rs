use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::anf::anf::ANFAtomic;
use crate::anf::anf::ANFDeclPattern;
use crate::anf::anf::ANFStatement;
use crate::anf::anf::ANFValue;
use crate::anf::anf::ANFVar;
use crate::anf::scc;
use crate::error::Span;
use crate::parser::core_expr::CoreDataConstructor;
use crate::parser::core_expr::CoreDeclPattern;
use crate::parser::core_expr::CoreExpr;
use crate::parser::core_expr::CoreMatchArm;
use crate::parser::core_expr::CoreMatchArmPattern;
use crate::parser::core_expr::CoreStatement;
use crate::parser::core_expr::CoreTupleDeclPattern;
use crate::parser::core_expr::CoreTypeExpr;

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

// --- metadata ---

pub struct ANFMetaData {
    pub name_by_id: HashMap<usize, String>,
}

// --- statement to ANFs ---

pub fn parse_anf(stmts: &Vec<CoreStatement>) -> (Vec<ANFStatement>, ANFMetaData) {
    let mut collected_anfs = Vec::new();
    let mut top_scope = Scope::new_root();

    // TODO: put built-in functions here. add extra ANF kind for built-in functions

    collect_stmts_to_anf(stmts, &mut top_scope, &mut collected_anfs);

    let metadata = ANFMetaData {
        name_by_id: top_scope.names.into_iter().map(|(k, v)| (v, k)).collect(),
    };

    (collected_anfs, metadata)
}

fn collect_stmts_to_anf(
    stmts: &Vec<CoreStatement>,
    scope: &mut Scope,
    collected_anfs: &mut Vec<ANFStatement>,
) {
    // -- name collection prepass

    let mut main_span = None;

    for stmt in stmts {
        match stmt {
            CoreStatement::VarDeclaration { pattern, .. } => {
                for name in pattern.all_identifiers() {
                    scope.register_name(name);
                }

                // check for main function
                match pattern {
                    CoreDeclPattern::Identifier { name, span } if name == "main" => {
                        main_span = Some(*span);
                    }
                    _ => {}
                }
            }

            CoreStatement::TypeDeclaration {
                expr: CoreTypeExpr::Sum { ctors, span },
                ..
            } => register_sum_type_ctors(ctors, scope, collected_anfs, *span),

            _ => {}
        }
    }

    // -- actual place where statements turn into ANFs

    for stmt in stmts {
        collect_stmt_to_anf(stmt, scope, collected_anfs);
    }

    // -- Tarjan's SCC

    // map declare LHS ids to their ANF expression
    let mut decl_stmt: HashMap<usize, usize> = HashMap::new();

    for (index, anf) in collected_anfs.iter().enumerate() {
        if let ANFStatement::Declaration(pattern, _) = anf {
            for id in pattern.all_ids() {
                decl_stmt.insert(id, index);
            }
        }
    }

    // build the dependency graph
    let mut dep_graph: HashMap<usize, Vec<usize>> =
        decl_stmt.keys().map(|&id| (id, Vec::new())).collect();

    for anf in collected_anfs.iter() {
        if let ANFStatement::Declaration(pattern, expr) = anf {
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
    let mut old_stmts: Vec<Option<ANFStatement>> = std::mem::take(collected_anfs)
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

    // insert main expr, if main were declared
    if let Some(span) = main_span {
        reordered.push(ANFStatement::Value(ANFValue::Atomic(ANFAtomic::Var {
            var: ANFVar {
                id: scope.get_id("main"),
                name: "main".to_string(),
            },
            span,
        })));
    }

    *collected_anfs = reordered;
}

fn register_sum_type_ctors(
    ctors: &Vec<CoreDataConstructor>,
    scope: &mut Scope,
    collected_anfs: &mut Vec<ANFStatement>,
    span: Span,
) {
    for ctor in ctors {
        let tag = ctor.tag.clone();
        let id = scope.register_name(&tag);

        let params: Vec<ANFVar> = ctor.types.iter().map(|_| scope.new_temp()).collect();

        let mut ctor_value = ANFValue::Atomic(ANFAtomic::Constructor {
            tag: tag.clone(),
            items: params
                .iter()
                .cloned()
                .map(|var| ANFValue::Atomic(ANFAtomic::Var { var, span }))
                .collect(),
            span,
        });

        for param in params.into_iter().rev() {
            ctor_value = ANFValue::Atomic(ANFAtomic::Lambda {
                param,
                body: Box::new(ctor_value),
                span,
            });
        }

        let ctor_decl_pattern = ANFDeclPattern::Single(ANFVar {
            id: Some(id),
            name: tag,
        });

        collected_anfs.push(ANFStatement::Declaration(ctor_decl_pattern, ctor_value));
    }
}

fn collect_stmt_to_anf(
    stmt: &CoreStatement,
    scope: &mut Scope,
    collected_anfs: &mut Vec<ANFStatement>,
) {
    match stmt {
        CoreStatement::VarDeclaration { pattern, expr, .. } => {
            let anf_expr = parse_expr_to_anf(expr, scope, collected_anfs);
            let anf_pattern = decl_pattern_to_anf(pattern, scope);
            let anf = ANFStatement::Declaration(anf_pattern, anf_expr.into());

            collected_anfs.push(anf);
        }

        CoreStatement::Expression { expr, .. } => {
            parse_expr_to_anf(expr, scope, collected_anfs);
        }

        _ => {}
    };
}

fn parse_expr_to_anf(
    expr: &CoreExpr,
    scope: &mut Scope,
    collected_anfs: &mut Vec<ANFStatement>,
) -> ANFValue {
    match expr {
        CoreExpr::Block { .. }
        | CoreExpr::Identifier { .. }
        | CoreExpr::Literal { .. }
        | CoreExpr::Lambda { .. }
        | CoreExpr::Tuple { .. }
        | CoreExpr::Match { .. } => {
            ANFValue::Atomic(parse_expr_to_atomic(expr, scope, collected_anfs))
        }

        CoreExpr::FunctionAppl { callee, arg, .. } => {
            let callee = parse_expr_to_atomic(callee, scope, collected_anfs);
            let arg = parse_expr_to_atomic(arg, scope, collected_anfs);

            ANFValue::FunctionAppl(callee, arg)
        }
    }
}

fn parse_expr_to_atomic(
    expr: &CoreExpr,
    scope: &mut Scope,
    collected_anfs: &mut Vec<ANFStatement>,
) -> ANFAtomic {
    match expr {
        // atomic -- trivial parse
        CoreExpr::Identifier { name, span } => ANFAtomic::Var {
            var: ANFVar {
                id: scope.get_id(name),
                name: name.to_string(),
            },
            span: *span,
        },

        CoreExpr::Literal { literal, span } => ANFAtomic::Literal {
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

            ANFAtomic::Lambda {
                param: ANFVar {
                    id: body_scope.get_id(param_name),
                    name: param_name.to_string(),
                },
                body: body_anf.into(),
                span: *span,
            }
        }

        CoreExpr::Tuple { items, span } => ANFAtomic::Tuple {
            items: items
                .into_iter()
                .map(|item| parse_expr_to_anf(item, scope, collected_anfs))
                .collect(),
            span: *span,
        },

        CoreExpr::Match {
            scrutinee,
            arms,
            span,
        } => ANFAtomic::Match {
            scrutinee: parse_expr_to_atomic(scrutinee, scope, collected_anfs).into(),
            arms: arms
                .iter()
                .map(|arm| parse_match_arm(arm, scope, span))
                .collect::<Vec<(_, _)>>(),
            span: *span,
        },

        CoreExpr::Block { statements, span } => {
            let mut block_collected_anf = Vec::new();

            let last_index = statements
                .iter()
                .position(|stmt| matches!(stmt, CoreStatement::Expression { .. }))
                .unwrap_or_else(|| {
                    unreachable!(
                        "missing final operand in `{}`. this should be catched in static check",
                        expr
                    )
                });

            for stmt in statements
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != last_index)
                .map(|(_, stmt)| stmt)
            {
                collect_stmt_to_anf(stmt, scope, &mut block_collected_anf);
            }

            let CoreStatement::Expression {
                expr: last_expr, ..
            } = &statements[last_index]
            else {
                unreachable!()
            };

            let last_anf = parse_expr_to_anf(last_expr, scope, &mut block_collected_anf);
            block_collected_anf.push(ANFStatement::Value(last_anf));

            ANFAtomic::Block {
                anfs: block_collected_anf,
                span: *span,
            }
        }

        CoreExpr::FunctionAppl { callee, arg, span } => {
            let callee = parse_expr_to_atomic(callee, scope, collected_anfs);
            let arg = parse_expr_to_atomic(arg, scope, collected_anfs);
            let anf = ANFValue::FunctionAppl(callee, arg);

            let var = scope.new_temp();
            let let_decl_pattern = ANFDeclPattern::Single(var.clone());
            collected_anfs.push(ANFStatement::Declaration(let_decl_pattern, anf));

            ANFAtomic::Var { var, span: *span }
        }
    }
}

fn parse_match_arm(
    arm: &CoreMatchArm,
    scope: &mut Scope,
    span: &Span,
) -> (CoreMatchArmPattern, ANFValue) {
    let pattern = arm.pattern.clone();

    let mut arm_collected_anf = Vec::new();
    let arm_anf = parse_expr_to_anf(&arm.value_expr, scope, &mut arm_collected_anf);
    let wrapped_arm_anf = wrap_scoped_anf(arm_collected_anf, arm_anf, *span);

    (pattern, wrapped_arm_anf)
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

fn wrap_scoped_anf(mut collected_anfs: Vec<ANFStatement>, tail: ANFValue, span: Span) -> ANFValue {
    if collected_anfs.is_empty() {
        tail
    } else {
        collected_anfs.push(ANFStatement::Value(tail));

        ANFValue::Atomic(ANFAtomic::Block {
            anfs: collected_anfs,
            span,
        })
    }
}
