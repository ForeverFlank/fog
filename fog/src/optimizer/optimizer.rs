use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::error::FogError;
use crate::parser::core_expr::CoreExpr;
use crate::parser::core_expr::CoreMatchArm;
use crate::parser::core_expr::CoreStatement;

// --- node ---

#[derive(Clone, PartialEq, Eq, Hash)]
enum Node {
    Statement(usize), // a top-level statement, by index in its block
    Name(String),     // a bound variable/type name
    Expr(u32),        // an expression
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct NodeId(usize);

impl NodeId {
    fn value(self) -> usize {
        self.0
    }
}

// --- union-find ---

struct DSU {
    parent: Vec<NodeId>,
    rank: Vec<usize>,
}

impl DSU {
    fn new() -> DSU {
        DSU {
            parent: Vec::new(),
            rank: Vec::new(),
        }
    }

    fn make_set(&mut self) -> NodeId {
        let id = NodeId(self.parent.len());
        self.parent.push(id);
        self.rank.push(0);

        id
    }

    fn find(&mut self, x: NodeId) -> NodeId {
        if self.parent[x.value()] != x {
            self.parent[x.value()] = self.find(self.parent[x.value()]);
        }

        self.parent[x.value()]
    }

    fn union(&mut self, a: NodeId, b: NodeId) {
        let (a, b) = (self.find(a), self.find(b));

        if a == b {
            return;
        }

        match self.rank[a.value()].cmp(&self.rank[b.value()]) {
            Ordering::Less => self.parent[a.value()] = b,
            Ordering::Greater => self.parent[b.value()] = a,
            Ordering::Equal => {
                self.parent[b.value()] = a;
                self.rank[a.value()] += 1;
            }
        }
    }
}

// --- dependency graph ---

struct DependencyGraph {
    dsu: DSU,
    id_by_node: HashMap<Node, NodeId>,
    edges: Vec<(NodeId, NodeId)>,
    expr_id_counter: u32,
}

impl DependencyGraph {
    fn new() -> DependencyGraph {
        DependencyGraph {
            dsu: DSU::new(),
            id_by_node: HashMap::new(),
            edges: Vec::new(),
            expr_id_counter: 0,
        }
    }

    fn add_node(&mut self, node: Node) -> NodeId {
        if let Some(&id) = self.id_by_node.get(&node) {
            return id;
        }

        let id = self.dsu.make_set();
        self.id_by_node.insert(node, id);

        id
    }

    fn add_stmt_node(&mut self, stmt_index: usize) -> NodeId {
        self.add_node(Node::Statement(stmt_index))
    }

    fn add_name_node(&mut self, name: String) -> NodeId {
        self.add_node(Node::Name(name))
    }

    fn add_expr_node(&mut self) -> NodeId {
        let id = self.expr_id_counter;
        self.expr_id_counter += 1;

        self.add_node(Node::Expr(id))
    }

    fn union(&mut self, a: NodeId, b: NodeId) {
        self.dsu.union(a, b);
    }

    fn add_edge(&mut self, scope: &Scope, from: NodeId, to_name: String) {
        if scope.is_unbound(&to_name) {
            return;
        }

        let to = self.add_node(Node::Name(to_name));
        self.edges.push((from, to));
    }

    fn to_adj_matrix(&mut self) -> Vec<Vec<NodeId>> {
        let node_count = self.dsu.parent.len();
        let mut adj = vec![Vec::new(); node_count];

        for i in 0..self.edges.len() {
            let (from, to) = self.edges[i];

            let from = self.dsu.find(from);
            let to = self.dsu.find(to);

            if from != to {
                adj[from.value()].push(to);
            }
        }

        adj
    }
}

// Tarjan's SCC algorithm
// https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm

fn tarjan_scc(adj: &Vec<Vec<NodeId>>) -> Vec<Vec<NodeId>> {
    let node_count = adj.len();

    let mut index = 0;
    let mut indices = vec![usize::MAX; node_count];
    let mut lowlinks = vec![usize::MAX; node_count];
    let mut on_stack = vec![false; node_count];
    let mut stack = Vec::new();

    let mut all_sccs = Vec::new();

    for v in 0..node_count {
        let v = NodeId(v);
        if indices[v.value()] == usize::MAX {
            strong_connect(
                v,
                adj,
                &mut index,
                &mut indices,
                &mut lowlinks,
                &mut stack,
                &mut on_stack,
                &mut all_sccs,
            );
        }
    }

    all_sccs
}

fn strong_connect(
    v: NodeId,
    adj: &Vec<Vec<NodeId>>,
    index: &mut usize,
    indices: &mut Vec<usize>,
    lowlinks: &mut Vec<usize>,
    stack: &mut Vec<NodeId>,
    on_stack: &mut Vec<bool>,
    all_sccs: &mut Vec<Vec<NodeId>>,
) {
    indices[v.value()] = *index;
    lowlinks[v.value()] = *index;
    *index += 1;

    stack.push(v);
    on_stack[v.value()] = true;

    for &w in &adj[v.value()] {
        if indices[w.value()] == usize::MAX {
            strong_connect(w, adj, index, indices, lowlinks, stack, on_stack, all_sccs);
            lowlinks[v.value()] = lowlinks[v.value()].min(lowlinks[w.value()]);
        } else if on_stack[w.value()] {
            lowlinks[v.value()] = lowlinks[v.value()].min(indices[w.value()]);
        }
    }

    if lowlinks[v.value()] == indices[v.value()] {
        let mut scc = Vec::new();

        loop {
            let w = stack.pop().unwrap();
            on_stack[w.value()] = false;
            scc.push(w);

            if v == w {
                break;
            }
        }

        all_sccs.push(scc);
    }
}

// --- scope ---

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

pub fn optimize(stmts: Vec<CoreStatement>) -> (Vec<CoreStatement>, Vec<FogError>) {
    optimize_block(stmts)
}

fn optimize_expr(expr: &mut CoreExpr, errors: &mut Vec<FogError>) {
    match expr {
        CoreExpr::Block { statements, .. } => {
            let taken = std::mem::take(statements);
            let (optimized, res_errors) = optimize_block(taken);

            *statements = optimized;
            errors.extend(res_errors);
        }

        CoreExpr::Lambda { body, .. } => {
            optimize_expr(body, errors);
        }

        CoreExpr::Tuple { items, .. } => {
            for item in items {
                optimize_expr(item, errors);
            }
        }

        CoreExpr::FunctionAppl { callee, arg, .. } => {
            optimize_expr(callee, errors);
            optimize_expr(arg, errors);
        }

        CoreExpr::Match {
            scrutinee, arms, ..
        } => {
            optimize_expr(scrutinee, errors);

            for arm in arms {
                optimize_expr(&mut arm.value_expr, errors);
            }
        }

        CoreExpr::Identifier { .. } | CoreExpr::Literal { .. } => {}
    }
}

fn optimize_block(mut stmts: Vec<CoreStatement>) -> (Vec<CoreStatement>, Vec<FogError>) {
    let optimized_stmts = Vec::new();
    let mut errors = Vec::new();

    // recursive optimize all nested expressions
    for stmt in &mut stmts {
        let expr = match stmt {
            CoreStatement::KindAnnotation { expr, .. }
            | CoreStatement::TypeDeclaration { expr, .. }
            | CoreStatement::TypeAnnotation { expr, .. }
            | CoreStatement::VarDeclaration { expr, .. }
            | CoreStatement::Expression { expr, .. } => expr,
        };

        optimize_expr(expr, &mut errors);
    }

    // build a graph
    let scope = Scope::root();
    let mut graph = DependencyGraph::new();

    for (index, stmt) in stmts.iter().enumerate() {
        visit_stmt(&mut graph, &scope, index, stmt);
    }

    // SCC
    let sccs = tarjan_scc(&graph.to_adj_matrix())
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    (optimized_stmts, errors)
}

fn visit_stmt(graph: &mut DependencyGraph, scope: &Scope, index: usize, stmt: &CoreStatement) {
    let stmt_node = graph.add_node(Node::Statement(index));

    match stmt {
        CoreStatement::KindAnnotation { name, expr, .. }
        | CoreStatement::TypeDeclaration { name, expr, .. }
        | CoreStatement::TypeAnnotation { name, expr, .. } => {
            let name_node = graph.add_name_node(name.clone());
            graph.union(stmt_node, name_node);

            visit_expr(graph, scope, stmt_node, expr);
        }

        CoreStatement::VarDeclaration { pattern, expr, .. } => {
            for name in pattern.all_identifiers() {
                let name_node = graph.add_name_node(name.to_string());
                graph.union(stmt_node, name_node);
            }

            visit_expr(graph, scope, stmt_node, expr);
        }

        CoreStatement::Expression { expr, .. } => {
            visit_expr(graph, scope, stmt_node, expr);
        }
    }
}

fn visit_stmt_expr(
    graph: &mut DependencyGraph,
    scope: &Scope,
    parent: NodeId,
    stmt: &CoreStatement,
) {
    let expr = match stmt {
        CoreStatement::KindAnnotation { expr, .. }
        | CoreStatement::TypeDeclaration { expr, .. }
        | CoreStatement::TypeAnnotation { expr, .. }
        | CoreStatement::VarDeclaration { expr, .. }
        | CoreStatement::Expression { expr, .. } => expr,
    };

    visit_expr(graph, scope, parent, expr);
}

fn visit_expr<'a>(
    graph: &mut DependencyGraph,
    scope: &'a Scope<'a>,
    parent: NodeId,
    expr: &CoreExpr,
) {
    let this = graph.add_expr_node();
    graph.union(parent, this);

    match expr {
        CoreExpr::Identifier { name, .. } => graph.add_edge(scope, this, name.clone()),

        CoreExpr::Block { statements, .. } => {
            for stmt in statements {
                visit_stmt_expr(graph, scope, this, stmt);
            }
        }

        CoreExpr::Lambda {
            param_name,
            param_type,
            body,
            ..
        } => {
            visit_expr(graph, scope, this, param_type);

            let unbound = HashSet::from([param_name.clone()]);
            let inner_scope = scope.child(unbound);

            visit_expr(graph, &inner_scope, this, body);
        }

        CoreExpr::Tuple { items, .. } => {
            for item in items {
                visit_expr(graph, scope, this, item);
            }
        }

        CoreExpr::FunctionAppl { callee, arg, .. } => {
            visit_expr(graph, scope, this, callee);
            visit_expr(graph, scope, this, arg);
        }

        CoreExpr::Match {
            scrutinee, arms, ..
        } => {
            visit_expr(graph, scope, this, scrutinee);

            for match_arm in arms {
                visit_match_arm(graph, scope, this, match_arm);
            }
        }

        CoreExpr::Literal { .. } => {}
    }
}

fn visit_match_arm<'a>(
    graph: &mut DependencyGraph,
    scope: &'a Scope<'a>,
    parent: NodeId,
    match_arm: &CoreMatchArm,
) {
    let unbound: HashSet<String> = match_arm
        .pattern
        .all_identifiers()
        .map(|name| name.to_string())
        .collect();

    let inner_scope = scope.child(unbound);

    visit_expr(graph, &inner_scope, parent, &match_arm.value_expr);
}
