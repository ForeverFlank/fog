use std::cmp::min;
use std::collections::HashMap;

use crate::error::FogError;
use crate::error::FogResult;
use crate::parser::core_expr::CoreDeclPattern;
use crate::parser::core_expr::CoreExpr;
use crate::parser::core_expr::CoreStatement;

// --- dependency graph ---

struct DependencyGraph {
    adj_list: HashMap<usize, Vec<usize>>,
    index_by_name: HashMap<String, usize>,
    name_by_index: Vec<String>,
    node_count: usize,
}

impl DependencyGraph {
    fn new() -> DependencyGraph {
        DependencyGraph {
            adj_list: HashMap::new(),
            index_by_name: HashMap::new(),
            name_by_index: Vec::new(),
            node_count: 0,
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

// --- optimizer ---
// currently it just does code sinking

pub fn optimize(stmts: Vec<CoreStatement>) -> (Vec<CoreStatement>, Vec<FogError>) {
    optimize_block(stmts)
}

fn optimize_block(stmts: Vec<CoreStatement>) -> (Vec<CoreStatement>, Vec<FogError>) {
    let mut optimized_stmts = Vec::new();
    let mut errors = Vec::new();

    let mut dep_graph = DependencyGraph::new();

    for stmt in stmts {
        add_dep_from_statements(&stmt, &mut dep_graph);
    }

    (optimized_stmts, errors)
}

fn add_dep_from_statements(stmt: &CoreStatement, dep_graph: &mut DependencyGraph) {
    match stmt {
        CoreStatement::TypeAnnotation { name, expr, span } => {
            add_dep_from_expr(name, expr, dep_graph);
        }

        // TODO: make declaration stronger?
        CoreStatement::Declaration {
            pattern,
            expr,
            span,
        } => match pattern {
            CoreDeclPattern::Identifier { name, span } => {
                add_dep_from_expr(name, expr, dep_graph);
            }

            CoreDeclPattern::Tuple { items, span } => match expr {
                CoreExpr::Block { statements, span } => {
                    for block_stmt in statements {
                        add_dep_from_statements(block_stmt, dep_graph);
                    }
                }

                CoreExpr::Identifier { name, span } => add_dep_from_expr(name, expr, dep_graph),

                CoreExpr::Tuple { items, span } => todo!(),

                _ => unreachable!(),
            },
        },

        CoreStatement::Expression { expr, span } => {}
    }
}

fn add_dep_from_expr(decl_name: &str, expr: &CoreExpr, dep_graph: &mut DependencyGraph) {
    match expr {
        CoreExpr::Block { statements, span } => {}

        CoreExpr::Identifier { name, span } => {
            dep_graph.add_dep(decl_name.to_string(), name.to_string())
        }

        CoreExpr::Literal { .. } => {}

        CoreExpr::Lambda { body, .. } => {
            add_dep_from_expr(decl_name, body.as_ref(), dep_graph);
        }

        CoreExpr::Tuple { items, .. } => {
            for item in items {
                add_dep_from_expr(decl_name, item, dep_graph);
            }
        }

        CoreExpr::FunctionAppl {
            fn_name,
            args,
            span,
        } => {
            dep_graph.add_dep(decl_name.to_string(), fn_name.to_string());

            for arg in args {
                add_dep_from_expr(decl_name, arg, dep_graph);
            }
        }

        CoreExpr::Match {
            scrutinee,
            match_arms,
            span,
        } => todo!(),
    }
}
