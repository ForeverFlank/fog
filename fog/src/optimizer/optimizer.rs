use std::collections::HashMap;

use crate::error::FogError;
use crate::parser::core_expr::CoreStatement;

// --- node ---

struct Node {
    id: usize,
    kind: NodeKind,
    ann_stmt: Option<CoreStatement>,
    decl_stmt: Option<CoreStatement>,
    expr_stmt: Option<CoreStatement>,
}

impl Node {
    fn new(id: usize, kind: NodeKind) -> Node {
        Node {
            id,
            kind,
            ann_stmt: None,
            decl_stmt: None,
            expr_stmt: None,
        }
    }
}

enum NodeKind {
    Decl(String),
    Result(usize),
}

// --- dependency graph ---

fn build_dependency_graph(stmts: Vec<CoreStatement>) {
    let mut nodes = HashMap::new();
    let mut result_nodes = Vec::new();

    for stmt in stmts {
        match stmt {
            CoreStatement::TypeAnnotation { name, expr, .. } => {
                nodes
                    .entry(name)
                    .or_insert(Node::new(0, NodeKind::Decl(name)))
                    .ann_stmt = Some(stmt);
            }

            CoreStatement::Declaration { pattern, expr, .. } => {
                for name in pattern.all_identifiers() {
                    let name_string = name.to_string();

                    nodes
                        .entry(name_string)
                        .or_insert(Node::new(0, NodeKind::Decl(name_string)))
                        .decl_stmt = Some(stmt);
                }
            }

            CoreStatement::Expression { expr, .. } => {
                result_nodes.push(Node::)
            },
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

    // let mut dep_graph = DependencyGraph::new();

    (optimized_stmts, errors)
}
