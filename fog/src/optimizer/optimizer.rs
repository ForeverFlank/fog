use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use crate::error::FogError;
use crate::parser::core_expr::CoreDeclPattern;
use crate::parser::core_expr::CoreExpr;
use crate::parser::core_expr::CoreStatement;
use crate::parser::core_expr::DesugaredMatchArm;

// --- node ---

#[derive(Clone)]
enum Node<'a> {
    TypeAnnotation {
        name: String,
        expr: &'a CoreExpr,
    },
    Declaration {
        names: Vec<String>,
        pattern: &'a CoreDeclPattern,
        expr: &'a CoreExpr,
    },
    Expression {
        expr: &'a CoreExpr,
    },
}

impl<'a> Node<'a> {
    fn new(stmt: &'a CoreStatement) -> Node<'a> {
        match stmt {
            CoreStatement::TypeAnnotation { name, expr, .. } => Node::TypeAnnotation {
                name: name.to_string(),
                expr,
            },
            CoreStatement::Declaration { pattern, expr, .. } => Node::Declaration {
                names: pattern.all_identifiers().map(|s| s.to_string()).collect(),
                pattern,
                expr,
            },
            CoreStatement::Expression { expr, .. } => Node::Expression { expr },
        }
    }
}

// --- dependency graph ---

struct DependencyGraph {
    adj_list: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    fn new() -> DependencyGraph {
        DependencyGraph {
            adj_list: HashMap::new(),
        }
    }

    fn add_edge(&mut self, scope: &Scope, from_names: &[&str], to_name: String) {
        if scope.is_unbound(&to_name) {
            return;
        }

        for from_name in from_names {
            self.adj_list
                .entry(from_name.to_string())
                .or_insert_with(Vec::new)
                .push(to_name.clone());
        }

        self.adj_list.entry(to_name).or_insert_with(Vec::new);
    }
}

/*
impl DependencyGraph {
    fn new() -> DependencyGraph {
        DependencyGraph {
            adj_list: HashMap::new(),
        }
    }

    fn node_index(&mut self, name: String) -> usize {
        if let Some(index) = self.index_by_name.get(&name) {
            *index
        } else {
            let index = self.node_count;
            self.index_by_name.insert(name.clone(), index);
            self.name_by_index.push(name);
            self.node_count += 1;

            index
        }
    }

    fn add_dep(&mut self, decl_name: String, ref_name: String) {
        let from_index = self.node_index(ref_name);
        let to_index = self.node_index(decl_name);

        self.adj_list
            .entry(from_index)
            .or_insert_with(Vec::new)
            .push(to_index);

        self.adj_list.entry(to_index).or_insert_with(Vec::new);
    }

    // Tarjan's SCC algorithm
    // from_names https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm

    fn toposort(&self) -> FogResult<Vec<Vec<String>>> {
        let mut index = 0;
        let mut indices = vec![usize::MAX; self.node_count];
        let mut lowlinks = vec![usize::MAX; self.node_count];

        let mut stack = Vec::new();
        let mut on_stack = vec![false; self.node_count];

        let mut all_sccs = Vec::new();

        for v in 0..self.node_count {
            if indices[v] == usize::MAX {
                Self::strong_connect(
                    v,
                    &self.adj_list,
                    &mut index,
                    &mut indices,
                    &mut lowlinks,
                    &mut stack,
                    &mut on_stack,
                    &mut all_sccs,
                );
            }
        }

        all_sccs.reverse();

        Ok(all_sccs
            .iter()
            .map(|scc| {
                scc.iter()
                    .map(|idx| self.name_by_index[*idx].clone())
                    .collect()
            })
            .collect())
    }

    fn strong_connect(
        v: usize,
        adj_list: &HashMap<usize, Vec<usize>>,
        index: &mut usize,
        indices: &mut Vec<usize>,
        lowlinks: &mut Vec<usize>,
        stack: &mut Vec<usize>,
        on_stack: &mut Vec<bool>,
        all_sccs: &mut Vec<Vec<usize>>,
    ) {
        indices[v] = *index;
        lowlinks[v] = *index;
        *index += 1;

        stack.push(v);
        on_stack[v] = true;

        for w in adj_list[&v].as_slice() {
            if indices[*w] == usize::MAX {
                Self::strong_connect(
                    *w, adj_list, index, indices, lowlinks, stack, on_stack, all_sccs,
                );
                lowlinks[v] = min(lowlinks[v], lowlinks[*w]);
            } else if on_stack[*w] {
                lowlinks[v] = min(lowlinks[v], indices[*w]);
            }
        }

        if lowlinks[v] == indices[v] {
            let mut scc = Vec::new();

            loop {
                let w = stack.pop().unwrap();
                on_stack[w] = false;
                scc.push(w);

                if v == w {
                    break;
                }
            }

            all_sccs.push(scc);
        }
    }
}
*/

struct Scope<'a> {
    parent: Option<&'a Scope<'a>>,
    unbound: HashSet<String>,
}

impl<'a> Scope<'a> {
    fn root() -> Scope<'a> {
        Scope {
            parent: None,
            unbound: HashSet::new(),
        }
    }

    fn child(&'a self, unbound: HashSet<String>) -> Scope<'a> {
        Scope {
            parent: Some(self),
            unbound,
        }
    }

    fn is_unbound(&self, name: &str) -> bool {
        self.unbound.contains(name) || self.parent.is_some_and(|p| p.is_unbound(name))
    }
}

// --- optimizer ---
// currently it just does code sinking

pub fn optimize(stmts: Vec<CoreStatement>) -> Result<Vec<CoreStatement>, Vec<FogError>> {
    optimize_block(stmts)
}

fn optimize_expr(expr: &mut CoreExpr, errors: &mut Vec<FogError>) {
    match expr {
        CoreExpr::Block { statements, .. } => {
            let taken = std::mem::take(statements);

            match optimize_block(taken) {
                Ok(optimized) => *statements = optimized,
                Err(mut errs) => errors.append(&mut errs),
            }
        }

        CoreExpr::Lambda { body, .. } => {
            optimize_expr(body, errors);
        }

        CoreExpr::Tuple { items, .. } => {
            for item in items {
                optimize_expr(item, errors);
            }
        }

        CoreExpr::FunctionAppl { args, .. } => {
            for arg in args {
                optimize_expr(arg, errors);
            }
        }

        CoreExpr::Match {
            scrutinee,
            match_arms,
            ..
        } => {
            optimize_expr(scrutinee, errors);

            for arm in match_arms {
                optimize_expr(&mut arm.value_expr, errors);
            }
        }

        CoreExpr::Identifier { .. } | CoreExpr::Literal { .. } => {}
    }
}

fn optimize_block(mut stmts: Vec<CoreStatement>) -> Result<Vec<CoreStatement>, Vec<FogError>> {
    let optimized_stmts = Vec::new();
    let mut errors = Vec::new();

    // recursive optimize all nested expressions
    for stmt in &mut stmts {
        let expr = match stmt {
            CoreStatement::TypeAnnotation { expr, .. }
            | CoreStatement::Declaration { expr, .. }
            | CoreStatement::Expression { expr, .. } => expr,
        };

        optimize_expr(expr, &mut errors);
    }

    // collect bound variable names
    let mut nodes = Vec::new();
    let mut node_by_name = HashMap::new();
    let mut bound_names = HashSet::new();

    for stmt in &stmts {
        let node = Rc::new(Node::new(stmt));
        nodes.push(node.clone());

        match stmt {
            CoreStatement::TypeAnnotation { name, .. } => {
                node_by_name.insert(name.to_string(), node.clone());
                bound_names.insert(name.to_string());
            }

            CoreStatement::Declaration { pattern, .. } => {
                for name in pattern.all_identifiers() {
                    node_by_name.insert(name.to_string(), node.clone());
                    bound_names.insert(name.to_string());
                }
            }

            CoreStatement::Expression { .. } => {}
        }
    }

    // build a graph
    let scope = Scope::root();
    let mut graph = DependencyGraph::new();

    for stmt in stmts {
        match stmt {
            CoreStatement::TypeAnnotation { name, expr, .. } => {
                let from_names = &vec![name.as_str()];
                build_dep_from_expr(&mut graph, &scope, from_names, expr)
            }

            CoreStatement::Declaration { pattern, expr, .. } => {
                let from_names = &pattern.all_identifiers().collect::<Vec<_>>();
                build_dep_from_expr(&mut graph, &scope, from_names, expr);
            }

            CoreStatement::Expression { .. } => {}
        }
    }

    if errors.is_empty() {
        Ok(optimized_stmts)
    } else {
        Err(errors)
    }
}

fn build_dep_from_stmt(
    graph: &mut DependencyGraph,
    scope: &Scope,
    from_names: &Vec<&str>,
    stmt: CoreStatement,
) {
    match stmt {
        CoreStatement::TypeAnnotation { expr, .. }
        | CoreStatement::Declaration { expr, .. }
        | CoreStatement::Expression { expr, .. } => {
            build_dep_from_expr(graph, scope, from_names, expr)
        }
    }
}

fn build_dep_from_expr<'a>(
    graph: &mut DependencyGraph,
    scope: &'a Scope<'a>,
    from_names: &Vec<&str>,
    expr: CoreExpr,
) {
    match expr {
        CoreExpr::Block { statements, .. } => {
            for stmt in statements {
                build_dep_from_stmt(graph, scope, from_names, stmt);
            }
        }

        CoreExpr::Identifier { name, .. } => graph.add_edge(scope, from_names, name),

        CoreExpr::Literal { .. } => {}

        CoreExpr::Lambda {
            param_name,
            param_type,
            body,
            ..
        } => {
            build_dep_from_expr(graph, scope, from_names, *param_type);

            let unbound = HashSet::from([param_name]);
            let inner_scope = scope.child(unbound);
            build_dep_from_expr(graph, &inner_scope, from_names, *body);
        }

        CoreExpr::Tuple { items, .. } => {
            for item in items {
                build_dep_from_expr(graph, scope, from_names, item);
            }
        }

        CoreExpr::FunctionAppl { fn_name, args, .. } => {
            graph.add_edge(scope, from_names, fn_name);

            for arg in args {
                build_dep_from_expr(graph, scope, from_names, arg);
            }
        }

        CoreExpr::Match {
            scrutinee,
            match_arms,
            ..
        } => {
            build_dep_from_expr(graph, scope, from_names, *scrutinee);

            for match_arm in match_arms {
                build_dep_from_match_arm(graph, scope, from_names, match_arm);
            }
        }
    }
}

fn build_dep_from_match_arm<'a>(
    graph: &mut DependencyGraph,
    scope: &'a Scope<'a>,
    from_names: &Vec<&str>,
    match_arm: DesugaredMatchArm,
) {
    let unbound: HashSet<String> = match_arm
        .pattern
        .all_identifiers()
        .map(|s| s.to_string())
        .collect();

    let inner_scope = scope.child(unbound);
    build_dep_from_expr(graph, &inner_scope, from_names, match_arm.value_expr);
}
