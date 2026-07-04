use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::io::ErrorKind::StaleNetworkFileHandle;

use crate::error::FogError;
use crate::parser::core_expr::{CoreExpr, CoreStatement};

// --- node ---

enum Node<'a> {
    TypeAnnotation {
        name: String,
        statement: &'a CoreStatement,
    },
    Declaration {
        name: String,
        statement: &'a CoreStatement,
    },
    ExprResult {
        statement: &'a CoreStatement,
    },
}

// --- scope ---

struct Scope<'a> {
    parent: Option<&'a Scope<'a>>,
    var_names: HashSet<String>,
    nodes: Vec<Node<'a>>,
}

impl<'a> Scope<'a> {
    fn new() -> Scope<'a> {
        Scope {
            parent: None,
            var_names: HashSet::new(),
            nodes: Vec::new(),
        }
    }

    fn with_parent(parent: &'a Scope<'a>) -> Scope<'a> {
        Scope {
            parent: Some(parent),
            var_names: HashSet::new(),
            nodes: Vec::new(),
        }
    }

    fn from_statements(parent: Option<&'a Scope<'a>>, stmts: &Vec<&'a CoreStatement>) -> Scope<'a> {
        let mut scope = Scope {
            parent,
            var_names: HashSet::new(),
            nodes: Vec::new(),
        };

        for &stmt in stmts {
            scope.add_statement(stmt);
        }

        let bound_vars = scope
            .nodes
            .iter()
            .filter_map(|node| match node {
                Node::TypeAnnotation { name, .. } => Some(name.to_string()),
                Node::Declaration { name, .. } => Some(name.to_string()),
                Node::ExprResult { .. } => None,
            })
            .collect::<HashSet<String>>();

        let mut dep_graph = DependencyGraph {
            adj_list: HashMap::new(),
        };

        for &stmt in stmts {
            match stmt {
                CoreStatement::TypeAnnotation { name, expr, span } => {
                    dep_graph.add_dep_by_expr(name, expr);
                }

                CoreStatement::Declaration {
                    pattern,
                    expr,
                    span,
                } => todo!(),

                CoreStatement::Expression { expr, span } => todo!(),
            }
        }

        scope
    }

    fn add_statement(&mut self, stmt: &'a CoreStatement) {
        let nodes = match stmt {
            CoreStatement::TypeAnnotation { name, .. } => vec![Node::TypeAnnotation {
                name: name.to_string(),
                statement: stmt,
            }],

            CoreStatement::Declaration { pattern, .. } => pattern
                .get_all_identifiers()
                .iter()
                .map(|name| Node::Declaration {
                    name: name.to_string(),
                    statement: stmt,
                })
                .collect(),

            CoreStatement::Expression { .. } => vec![Node::ExprResult { statement: stmt }],
        };

        self.nodes.extend(nodes);
    }
}

// --- dependency graph ---

struct DependencyGraph {
    adj_list: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    fn add_dep_by_name(&mut self, decl_name: &str, depending_on_name: &str) {
        self.adj_list
            .entry(decl_name.to_string())
            .or_insert_with(Vec::new)
            .push(depending_on_name.to_string());

        self.adj_list
            .entry(depending_on_name.to_string())
            .or_insert_with(Vec::new);
    }

    fn add_dep_by_expr(&mut self, decl_name: &str, depending_on_expr: &CoreExpr) {
        match depending_on_expr {
            CoreExpr::Block { statements, span } => todo!(),
            CoreExpr::Identifier { name, span } => todo!(),
            CoreExpr::Literal { literal, span } => todo!(),
            CoreExpr::Lambda {
                param_name,
                param_type,
                body,
                span,
            } => todo!(),
            CoreExpr::Tuple { items, span } => todo!(),
            CoreExpr::FunctionAppl {
                fn_name,
                args,
                span,
            } => todo!(),
            CoreExpr::Match {
                scrutinee,
                match_arms,
                span,
            } => todo!(),
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

pub fn optimize(stmts: Vec<CoreStatement>) -> (Vec<CoreStatement>, Vec<FogError>) {
    optimize_block(stmts)
}

fn optimize_block(stmts: Vec<CoreStatement>) -> (Vec<CoreStatement>, Vec<FogError>) {
    let mut optimized_stmts = Vec::new();
    let mut errors = Vec::new();

    (optimized_stmts, errors)
}
