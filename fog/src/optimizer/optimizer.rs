use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::error::FogError;
use crate::parser::core_expr::CoreExpr;
use crate::parser::core_expr::CoreStatement;
use crate::parser::core_expr::DesugaredMatchArm;

// --- node ---

#[derive(Clone, PartialEq, Eq, Hash)]
enum Node {
    Stmt(usize),  // a top-level statement, by index in its block
    Name(String), // a bound variable/type name
    Expr(u32),    // an expression
}

// --- union-find ---

struct DisjointSet {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl DisjointSet {
    fn new() -> DisjointSet {
        DisjointSet {
            parent: Vec::new(),
            rank: Vec::new(),
        }
    }

    fn make_set(&mut self) -> usize {
        let id = self.parent.len();
        self.parent.push(id);
        self.rank.push(0);
        id
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }

        self.parent[x]
    }

    fn union(&mut self, a: usize, b: usize) {
        let (a, b) = (self.find(a), self.find(b));

        if a == b {
            return;
        }

        match self.rank[a].cmp(&self.rank[b]) {
            Ordering::Less => self.parent[a] = b,
            Ordering::Greater => self.parent[b] = a,
            Ordering::Equal => {
                self.parent[b] = a;
                self.rank[a] += 1;
            }
        }
    }
}

// --- dependency graph ---

struct GraphBuilder {
    dsu: DisjointSet,
    id_by_key: HashMap<Node, usize>,
    edges: Vec<(usize, usize)>,
    expr_id_counter: u32,
}

impl GraphBuilder {
    fn new() -> GraphBuilder {
        GraphBuilder {
            dsu: DisjointSet::new(),
            id_by_key: HashMap::new(),
            expr_id_counter: 0,
            edges: Vec::new(),
        }
    }

    fn node(&mut self, key: Node) -> usize {
        if let Some(&id) = self.id_by_key.get(&key) {
            return id;
        }

        let id = self.dsu.make_set();
        self.id_by_key.insert(key, id);

        id
    }

    fn fresh_expr_node(&mut self) -> usize {
        let id = self.expr_id_counter;
        self.expr_id_counter += 1;

        self.node(Node::Expr(id))
    }

    fn union(&mut self, a: usize, b: usize) {
        self.dsu.union(a, b);
    }

    fn add_edge(&mut self, scope: &Scope, from: usize, to_name: String) {
        if scope.is_unbound(&to_name) {
            return;
        }

        let to = self.node(Node::Name(to_name));
        self.edges.push((from, to));
    }
}

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

    // build a graph
    let scope = Scope::root();
    let mut graph_builder = GraphBuilder::new();

    for (index, stmt) in stmts.iter().enumerate() {
        visit_stmt(&mut graph_builder, &scope, index, stmt);
    }

    // toposort
    // etc

    if errors.is_empty() {
        Ok(optimized_stmts)
    } else {
        Err(errors)
    }
}

fn visit_stmt(builder: &mut GraphBuilder, scope: &Scope, index: usize, stmt: &CoreStatement) {
    let stmt_node = builder.node(Node::Stmt(index));

    match stmt {
        CoreStatement::TypeAnnotation { name, expr, .. } => {
            let name_node = builder.node(Node::Name(name.clone()));
            builder.union(stmt_node, name_node);

            visit_expr(builder, scope, stmt_node, expr);
        }

        CoreStatement::Declaration { pattern, expr, .. } => {
            for name in pattern.all_identifiers() {
                let name_node = builder.node(Node::Name(name.to_string()));
                builder.union(stmt_node, name_node);
            }

            visit_expr(builder, scope, stmt_node, expr);
        }

        CoreStatement::Expression { expr, .. } => {
            visit_expr(builder, scope, stmt_node, expr);
        }
    }
}

fn visit_stmt_expr(builder: &mut GraphBuilder, scope: &Scope, parent: usize, stmt: &CoreStatement) {
    let expr = match stmt {
        CoreStatement::TypeAnnotation { expr, .. }
        | CoreStatement::Declaration { expr, .. }
        | CoreStatement::Expression { expr, .. } => expr,
    };

    visit_expr(builder, scope, parent, expr);
}

fn visit_expr<'a>(
    builder: &mut GraphBuilder,
    scope: &'a Scope<'a>,
    parent: usize,
    expr: &CoreExpr,
) {
    let this = builder.fresh_expr_node();
    builder.union(parent, this);

    match expr {
        CoreExpr::Block { statements, .. } => {
            for stmt in statements {
                visit_stmt_expr(builder, scope, this, stmt);
            }
        }

        CoreExpr::Identifier { name, .. } => builder.add_edge(scope, this, name.clone()),

        CoreExpr::Literal { .. } => {}

        CoreExpr::Lambda {
            param_name,
            param_type,
            body,
            ..
        } => {
            visit_expr(builder, scope, this, param_type);

            let unbound = HashSet::from([param_name.clone()]);
            let inner_scope = scope.child(unbound);
            visit_expr(builder, &inner_scope, this, body);
        }

        CoreExpr::Tuple { items, .. } => {
            for item in items {
                visit_expr(builder, scope, this, item);
            }
        }

        CoreExpr::FunctionAppl { fn_name, args, .. } => {
            builder.add_edge(scope, this, fn_name.clone());

            for arg in args {
                visit_expr(builder, scope, this, arg);
            }
        }

        CoreExpr::Match {
            scrutinee,
            match_arms,
            ..
        } => {
            visit_expr(builder, scope, this, scrutinee);

            for match_arm in match_arms {
                visit_match_arm(builder, scope, this, match_arm);
            }
        }
    }
}

fn visit_match_arm<'a>(
    builder: &mut GraphBuilder,
    scope: &'a Scope<'a>,
    parent: usize,
    match_arm: &DesugaredMatchArm,
) {
    let unbound: HashSet<String> = match_arm
        .pattern
        .all_identifiers()
        .map(|name| name.to_string())
        .collect();

    let inner_scope = scope.child(unbound);
    visit_expr(builder, &inner_scope, parent, &match_arm.value_expr);
}
