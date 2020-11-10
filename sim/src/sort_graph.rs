
use super::types::*;

enum VisitState {
    /*
     * Stores the address, the dependencies
     * and the state of this edge for the current visit.
     */
    NotVisited(usize, Vec<usize>, VarDef),
    InProgress,
    Visited,
}

use VisitState::*;

struct Visitor {
    edges: Vec<VisitState>,
    sorted: Vec<(usize, VarDef)>,
}

impl Visitor {
    fn new(edges: Vec<VarInfo>) -> Self {
        let len = edges.len();
        Visitor {
            edges: edges.into_iter().map(|info| NotVisited(info.address, info.deps, info.def)).collect(),
            sorted: Vec::with_capacity(len),
        }
    }

    fn visit(&mut self, edge: usize) -> Result<(), ()> {
        let state = &mut self.edges[edge];

        match *state {
            NotVisited(_, _, _) => {
                let (address, deps, def) = match std::mem::replace(state, InProgress) {
                    NotVisited(address, deps, def) => (address, deps, def),
                    _ => panic!()
                };

                deps.clone().iter().try_for_each(|id|
                        self.visit(*id)
                    )?;
                self.edges[edge] = Visited;
                self.sorted.push((address, def));

                Ok(())
            },
            InProgress => Err(()),
            Visited => Ok(())
        }
    }
}

pub fn sort_graph(graph: OpsGraph) -> Result<OpsList, String> {
    let len = graph.edges.len();
    let mut visitor = Visitor::new(graph.edges);

    (0..len).try_for_each(|id| visitor.visit(id))
        .map_err(|_| "Circular dependency in variables.".to_string())?;

    Ok(OpsList {
        mem_size: graph.mem_size,
        inputs: graph.inputs,
        outputs: graph.outputs,
        mems: graph.mems,
        ops: visitor.sorted,
        mem_ops: graph.mem_ops,
    })
}

