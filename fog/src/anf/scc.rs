// Tarjan's Strongly Connected Components algorithm
// from https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm

// algorithm tarjan is
//     input: graph G = (V, E)
//     output: set of strongly connected components (sets of vertices)

//     index := 0
//     S := empty stack
//     for each v in V do
//         if v.index is undefined then
//             strongconnect(v)

//     function strongconnect(v)
//         // Set the depth index for v to the smallest unused index
//         v.index := index
//         v.lowlink := index
//         index := index + 1
//         S.push(v)
//         v.onStack := true

//         // Consider successors of v
//         for each (v, w) in E do
//             if w.index is undefined then
//                 // Successor w has not yet been visited; recurse on it
//                 strongconnect(w)
//                 v.lowlink := min(v.lowlink, w.lowlink)
//             else if w.onStack then
//                 // Successor w is in stack S and hence in the current SCC
//                 // If w is not on stack, then (v, w) is an edge pointing to an SCC already found and must be ignored
//                 // See below regarding the next line
//                 v.lowlink := min(v.lowlink, w.index)

//         // If v is a root node, pop the stack and generate an SCC
//         if v.lowlink = v.index then
//             start a new strongly connected component
//             repeat
//                 w := S.pop()
//                 w.onStack := false
//                 add w to current strongly connected component
//             while w ≠ v
//             output the current strongly connected component

fn tarjan_scc(adj: Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    let num_indices = adj.len();

    let mut index = 0usize;
    let mut stack: Vec<usize> = Vec::new();

    let mut indices = vec![usize::MAX; num_indices];
    let mut lowlinks = vec![usize::MAX; num_indices];
    let mut on_stack = vec![false; num_indices];

    let mut sccs = Vec::new();

    for v in 0..num_indices {
        if indices[v] == usize::MAX {
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
    adj: &Vec<Vec<usize>>,
    sccs: &mut Vec<Vec<usize>>,
    index: &mut usize,
    indices: &mut Vec<usize>,
    lowlinks: &mut Vec<usize>,
    stack: &mut Vec<usize>,
    on_stack: &mut Vec<bool>,
) {
    indices[v] = *index;
    lowlinks[v] = *index;
    *index += 1;
    stack.push(v);
    on_stack[v] = true;

    for w in adj[v].iter() {
        let w = *w;

        if indices[w] == usize::MAX {
            strongconnect(w, adj, sccs, index, indices, lowlinks, stack, on_stack);
            lowlinks[v] = std::cmp::min(lowlinks[v], lowlinks[w]);
        } else if on_stack[w] {
            lowlinks[v] = std::cmp::min(lowlinks[v], indices[w]);
        }
    }

    if lowlinks[v] == indices[v] {
        let mut scc = Vec::new();

        loop {
            let w = stack.pop().unwrap();
            on_stack[w] = false;
            scc.push(w);

            if w == v {
                break;
            }
        }

        sccs.push(scc);
    }
}
