use std::collections::HashMap;
use std::collections::HashSet;

// Tarjan's Strongly Connected Components algorithm
// from https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm

pub fn tarjan_scc(adj: HashMap<usize, Vec<usize>>) -> Vec<Vec<usize>> {
    let mut index: usize = 0;
    let mut stack: Vec<usize> = Vec::new();

    let mut indices: HashMap<usize, usize> = HashMap::new();
    let mut lowlinks: HashMap<usize, usize> = HashMap::new();
    let mut on_stack: HashSet<usize> = HashSet::new();

    let mut sccs = Vec::new();

    for &v in adj.keys() {
        if !indices.contains_key(&v) {
            strongconnect(
                v,
                &adj,
                &mut sccs,
                &mut index,
                &mut indices,
                &mut lowlinks,
                &mut stack,
                &mut on_stack,
            );
        }
    }

    sccs
}

fn strongconnect(
    v: usize,
    adj: &HashMap<usize, Vec<usize>>,
    sccs: &mut Vec<Vec<usize>>,
    index: &mut usize,
    indices: &mut HashMap<usize, usize>,
    lowlinks: &mut HashMap<usize, usize>,
    stack: &mut Vec<usize>,
    on_stack: &mut HashSet<usize>,
) {
    indices.insert(v, *index);
    lowlinks.insert(v, *index);
    *index += 1;
    stack.push(v);
    on_stack.insert(v);

    for &w in adj.get(&v).into_iter().flatten() {
        if !indices.contains_key(&w) {
            strongconnect(w, adj, sccs, index, indices, lowlinks, stack, on_stack);
            lowlinks.insert(v, std::cmp::min(lowlinks[&v], lowlinks[&w]));
        } else if on_stack.contains(&w) {
            lowlinks.insert(v, std::cmp::min(lowlinks[&v], indices[&w]));
        }
    }

    if lowlinks[&v] == indices[&v] {
        let mut scc = Vec::new();

        loop {
            let w = stack.pop().unwrap();
            on_stack.remove(&w);
            scc.push(w);

            if w == v {
                break;
            }
        }

        sccs.push(scc);
    }
}
