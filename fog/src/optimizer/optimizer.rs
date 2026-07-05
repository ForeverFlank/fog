use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use crate::error::FogError;
use crate::parser::core_expr::CoreDeclPattern;
use crate::parser::core_expr::CoreExpr;
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
}

impl DependencyGraph {
    fn add_edge(&mut self, from: &str, to: String) {
        self.adj_list
            .entry(from.to_string())
            .or_insert_with(Vec::new)
            .push(to.clone());

        self.adj_list.entry(to).or_insert_with(Vec::new);
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
    // from https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm

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
    let mut bound_vars = HashSet::new();

    for stmt in &stmts {
        let node = Rc::new(Node::new(stmt));
        nodes.push(node.clone());

        match stmt {
            CoreStatement::TypeAnnotation { name, .. } => {
                node_by_name.insert(name.to_string(), node.clone());
                bound_vars.insert(name.to_string());
            }

            CoreStatement::Declaration { pattern, .. } => {
                for name in pattern.all_identifiers() {
                    node_by_name.insert(name.to_string(), node.clone());
                    bound_vars.insert(name.to_string());
                }
            }

            CoreStatement::Expression { .. } => {}
        }
    }

    // build a graph
    let mut dep_graph = DependencyGraph {
        adj_list: HashMap::new(),
    };

    for stmt in stmts {
        match stmt {
            CoreStatement::TypeAnnotation { name, expr, span } => {
                build_dep_from_expr(&mut dep_graph, name, expr)
            }
            CoreStatement::Declaration {
                pattern,
                expr,
                span,
            } => todo!(),
            CoreStatement::Expression { expr, span } => todo!(),
        }
    }

    if errors.is_empty() {
        Ok(optimized_stmts)
    } else {
        Err(errors)
    }
}

fn build_dep_from_stmt(dep_graph: &mut DependencyGraph, from: &str, stmt: CoreStatement) {
    match stmt {
        CoreStatement::TypeAnnotation { expr, .. }
        | CoreStatement::Declaration { expr, .. }
        | CoreStatement::Expression { expr, .. } => build_dep_from_expr(dep_graph, from, expr),
    }
}

fn build_dep_from_expr(dep_graph: &mut DependencyGraph, from: &str, expr: CoreExpr) {
    match expr {
        CoreExpr::Block { statements, .. } => {
            for stmt in statements {
                build_dep_from_stmt(dep_graph, from, stmt);
            }
        }

        CoreExpr::Identifier { name, .. } => dep_graph.add_edge(from, name),

        CoreExpr::Literal { .. } => {}

        CoreExpr::Lambda {
            param_name,
            param_type,
            body,
            ..
        } => todo!(),
        CoreExpr::Tuple { items, .. } => todo!(),
        CoreExpr::FunctionAppl { fn_name, args, .. } => todo!(),
        CoreExpr::Match {
            scrutinee,
            match_arms,
            ..
        } => todo!(),
    }
}
