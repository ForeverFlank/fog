use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use crate::error::FogError;
use crate::parser::core_expr::CoreDeclPattern;
use crate::parser::core_expr::CoreExpr;
use crate::parser::core_expr::CoreMatchArmPattern;
use crate::parser::core_expr::CoreStatement;

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
    bound_names: HashSet<String>,
}

impl DependencyGraph {
    fn new(bound_names: HashSet<String>) -> DependencyGraph {
        DependencyGraph {
            adj_list: HashMap::new(),
            bound_names,
        }
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
    bounded_names: HashSet<String>,
    unbounded_names: HashSet<String>,
    adj_list: HashMap<String, Vec<String>>,
}

impl<'a> Scope<'a> {
    fn new(parent: Option<&'a Scope<'a>>, bounded_names: HashSet<String>) -> Scope<'a> {
        Scope {
            parent,
            bounded_names,
            unbounded_names: HashSet::new(),
            adj_list: HashMap::new(),
        }
    }

    fn add_edge(&mut self, from_names: &Vec<&str>, to_name: String) {
        if self.unbounded_names.contains(&to_name) {
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

// --- optimizer ---
// currently it just does code sinking

pub fn optimize(stmts: Vec<CoreStatement>) -> Result<Vec<CoreStatement>, Vec<FogError>> {
    optimize_block(stmts)
}

fn optimize_block(stmts: Vec<CoreStatement>) -> Result<Vec<CoreStatement>, Vec<FogError>> {
    let mut optimized_stmts = Vec::new();
    let mut errors = Vec::new();

    // optimize children block statements
    for stmt in &stmts {
        match stmt {
            CoreStatement::Declaration { expr, .. } | CoreStatement::Expression { expr, .. } => {
                if let CoreExpr::Block { statements, .. } = expr {
                    // optimize_block(statements); // TODO replace stmt in place
                }
            }

            CoreStatement::TypeAnnotation { .. } => {}
        }
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
    let mut scope = Scope::new(None, bound_names);

    for stmt in stmts {
        match stmt {
            CoreStatement::TypeAnnotation { name, expr, .. } => {
                let from_names = &vec![name.as_str()];

                build_dep_from_expr(&mut scope, from_names, expr)
            }

            CoreStatement::Declaration { pattern, expr, .. } => {
                let from_names = &pattern.all_identifiers().collect::<Vec<_>>();

                build_dep_from_expr(&mut scope, from_names, expr);
            }

            CoreStatement::Expression { expr, .. } => todo!(),
        }
    }

    if errors.is_empty() {
        Ok(optimized_stmts)
    } else {
        Err(errors)
    }
}

fn build_dep_from_stmt(scope: &mut Scope, from_names: &Vec<&str>, stmt: CoreStatement) {
    match stmt {
        CoreStatement::TypeAnnotation { expr, .. }
        | CoreStatement::Declaration { expr, .. }
        | CoreStatement::Expression { expr, .. } => build_dep_from_expr(scope, from_names, expr),
    }
}

fn build_dep_from_expr(scope: &mut Scope, from_names: &Vec<&str>, expr: CoreExpr) {
    match expr {
        CoreExpr::Block { statements, .. } => {
            for stmt in statements {
                build_dep_from_stmt(scope, from_names, stmt);
            }
        }

        CoreExpr::Identifier { name, .. } => scope.add_edge(from_names, name),

        CoreExpr::Literal { .. } => {}

        CoreExpr::Lambda {
            param_name,
            param_type,
            body,
            ..
        } => {
            build_dep_from_expr(scope, from_names, *param_type);

            scope.unbounded_names.insert(param_name.clone());
            build_dep_from_expr(scope, from_names, *body);
            scope.unbounded_names.remove(&param_name);
        }

        CoreExpr::Tuple { items, .. } => {
            for item in items {
                build_dep_from_expr(scope, from_names, item);
            }
        }

        CoreExpr::FunctionAppl { fn_name, args, .. } => {
            scope.add_edge(from_names, fn_name);

            for arg in args {
                build_dep_from_expr(scope, from_names, arg);
            }
        }

        CoreExpr::Match {
            scrutinee,
            match_arms,
            ..
        } => {
            build_dep_from_expr(scope, from_names, *scrutinee);

            for match_arm in match_arms {
                match match_arm.pattern {
                    CoreMatchArmPattern::Tuple { items, span } => {
                        scope.unbounded_names.remove(&items); // or smth like that
                        build_dep_from_expr(scope, from_names, *body);
                        scope.unbounded_names.remove(&items);
                    }

                    CoreMatchArmPattern::Identifier { name, span } => {
                        scope.unbounded_names.insert(name.clone());
                        build_dep_from_expr(scope, from_names, *body);
                        scope.unbounded_names.remove(&name);
                    }

                    CoreMatchArmPattern::DataConstructor { name, args, span } => {}

                    CoreMatchArmPattern::Literal { literal, span } => {}
                }
            }
        }
    }
}
