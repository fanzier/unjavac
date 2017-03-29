//! Convert a control flow to structured control flow statements (if, while, break, continue).
//!
//! It uses the following strategy:
//!
//! 1. Find strongly connected components, i.e. loops
//! 2. Check each loop has at most one entry point and
//!    find the exit point (postdominator of the exit points)
//! 3. For each loop: replace all edges to the entry point from within the loop
//!    by `continue $current_loop_label;`
//! 4. For each loop replace all edges to the exit point from within the loop
//!    by `break $current_loop_label;`
//! 5. Recursively structure the loop content and wrap it in `while(true) { ... }`
//! 6. We now have an acyclic control flow graph.
//! 7. run `structure_from(#method_entry_point, None)`, see below
//!
//! ```
//! structure_from(#start, #stop):
//!   if #start == #stop:
//!     stop
//!   convert and output #start
//!   if #start has at most one successor #successor:
//!     structure_from(#successor)
//!   if #start branches into #then and #else:
//!     #join := postdominator of #start
//!     output "if $branch_condition"
//!     structure_from(#then, #join)
//!     output "else"
//!     structure_from(#else, #join);
//!     structure_from(#join, #stop);
//!     structure_from(#else, #join)
//!  ```
//!
//! The actual implementation is less recursive in order not to run the risk of blowing the stack.
//! (If only Rust had guaranteed tail call optimization...)

use disassembler::types::*;
use decompiler::cfg::*;
use decompiler::types::*;
use std::collections::{BTreeSet, BTreeMap};

type Set<T> = BTreeSet<T>;
type Map<K, T> = BTreeMap<K, T>;

pub fn structure(unit: CompilationUnit<Cfg<Statement, RecExpr>>) -> CompilationUnit<Block> {
    unit.map(structure_cfg)
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct Loop {
    nodes: Set<Label>,
    entry: Label,
    exits: Set<Label>,
}

fn loop_label(index: usize) -> String {
    format!("loop_{}", index)
}

#[derive(Debug)]
struct Context {
    cfg: Cfg<Statement, RecExpr>,
    loops: Vec<Loop>,
    entry_to_loop_index: Map<Label, usize>,
}

fn create_context(cfg: Cfg<Statement, RecExpr>) -> Context {
    Context {
        cfg: cfg,
        loops: vec![],
        entry_to_loop_index: Map::new(),
    }
}

fn structure_cfg(cfg: Cfg<Statement, RecExpr>, _: &Metadata) -> Block {
    let mut ctx = create_context(cfg);
    let all_nodes = ctx.cfg
        .graph
        .node_indices()
        .collect::<Set<_>>();
    collect_loops(&mut ctx, &all_nodes);
    let entry = ctx.cfg.entry_point;
    let result = structure_from_to(&mut ctx, entry, None);
    Block(vec![], result)
}

fn structure_from_to(ctx: &mut Context, mut cur: Label, stop: Option<Label>) -> Vec<Statement> {
    let mut result = vec![];
    while Some(cur) != stop {
        let next = translate_block(ctx, &mut result, cur, None);
        if let Some(next) = next {
            cur = next;
        } else {
            break;
        }
    }
    result
}

fn handle_jump(ctx: &mut Context,
               result: &mut Vec<Statement>,
               cur: Label,
               next: Label)
               -> Option<Label> {
    if let Some(&loop_index) = ctx.entry_to_loop_index.get(&next) {
        let is_continue_jump = ctx.loops[loop_index].nodes.contains(&cur);
        let label = loop_label(loop_index);
        if is_continue_jump {
            result.push(Statement::Continue(Some(label)));
            return None;
        } else {
            let body = structure_from_to(ctx, next, None); // TODO use better stop point
            result.push(Statement::While {
                            label: Some(label),
                            cond: mk_variable("true".into()),
                            body: Block(vec![], body),
                            do_while: false,
                        });
            return None; // TODO use stop point from above
        }
    } else {
        return Some(next);
    }
}

fn translate_block(ctx: &mut Context,
                   result: &mut Vec<Statement>,
                   cur: Label,
                   stop: Option<Label>)
                   -> Option<Label> {
    let mut outgoing: Map<Edge, Label> = Map::new();
    for edge in ctx.cfg.graph.edges_directed(cur, Direction::Outgoing) {
        outgoing.insert(*edge.weight(), edge.target());
    }
    let mut bb = ctx.cfg.graph[cur].clone();
    result.append(&mut bb.stmts);
    let cond = bb.terminator;
    if let Some(cond) = cond {
        assert_eq!(2,
                   outgoing.len(),
                   "basic block #{} (with condition) should have 2 successors",
                   cur.index());
        // TODO: find join point
        let mut then_stmts = vec![];
        let then_block = handle_jump(ctx, &mut then_stmts, cur, outgoing[&true]);
        then_stmts.append(&mut then_block.map_or_else(Vec::new,
                                                      |then| structure_from_to(ctx, then, stop)));
        let mut else_stmts = vec![];
        let else_block = handle_jump(ctx, &mut else_stmts, cur, outgoing[&false]);
        else_stmts.append(&mut else_block.map_or_else(Vec::new,
                                                      |els| structure_from_to(ctx, els, stop)));
        result.push(Statement::If {
                        cond: cond,
                        then: Block(vec![], then_stmts),
                        els: Some(Block(vec![], else_stmts)),
                    });
        None
    } else {
        assert!(outgoing.len() <= 1,
                "basic block #{} has too many successors",
                cur.index());
        if let Some(next) = outgoing.values().next() {
            if let Some(next) = handle_jump(ctx, result, cur, *next) {
                return Some(next);
            }
        }
        None
    }
}

fn collect_loops(ctx: &mut Context, filter: &Set<Label>) {
    if filter.is_empty() {
        return;
    }
    let sccs = compute_strongly_connected_components(&ctx.cfg.graph, filter);
    for mut nodes in sccs {
        if !is_scc_loop(ctx, &nodes) {
            continue;
        }
        let (entry_points, exit_points) = find_entries_and_exits(ctx, &nodes);
        // extract the entry point:
        assert!(entry_points.len() <= 1,
                "The loop consisting of the basic blocks {:?} has multiple entry points {:?}",
                nodes,
                entry_points);
        if let Some(entry_point) = entry_points.iter().cloned().next() {
            store_loop_in_context(ctx, &nodes, entry_point, exit_points);
            // recursively collect the nested loops inside this loop:
            nodes.remove(&entry_point);
            collect_loops(ctx, &nodes);
        }
    }
}

fn compute_strongly_connected_components(graph: &CfgGraph<Statement, RecExpr>,
                                         filter: &Set<Label>)
                                         -> Vec<Set<Label>> {
    let filtered = NodeFiltered(graph, |n| filter.contains(&n));
    algo::kosaraju_scc(&filtered)
    .iter_mut()
    .rev() // we want a topological sort
    .map(|v| v.drain(..).collect::<Set<_>>())
    .collect::<Vec<_>>()
}

fn is_scc_loop(ctx: &Context, nodes: &Set<Label>) -> bool {
    if nodes.is_empty() {
        return false;
    }
    if nodes.len() == 1 {
        let node = *nodes.iter().next().unwrap();
        ctx.cfg
            .graph
            .find_edge(node, node)
            .is_some()
    } else {
        true
    }
}

fn find_entries_and_exits(ctx: &Context, nodes: &Set<Label>) -> (Set<Label>, Set<Label>) {
    let mut entry_points = Set::new();
    let mut exit_points = Set::new();
    for &node in nodes {
        let graph = &ctx.cfg.graph;
        let incoming_neighbors = graph.neighbors_directed(node, Direction::Incoming);
        for incoming in incoming_neighbors {
            if !nodes.contains(&incoming) {
                // not an intra-loop edge
                entry_points.insert(node);
            }
        }
        let outgoing_neighbors = graph.neighbors_directed(node, Direction::Outgoing);
        for outgoing in outgoing_neighbors {
            if !nodes.contains(&outgoing) {
                // not an intra-loop edge
                exit_points.insert(outgoing);
            }
        }
    }
    (entry_points, exit_points)
}

fn store_loop_in_context(ctx: &mut Context, nodes: &Set<Label>, entry: Label, exits: Set<Label>) {
    let lupe = Loop {
        nodes: nodes.clone(),
        entry: entry,
        exits: exits,
    };
    ctx.loops.push(lupe);
    ctx.entry_to_loop_index.insert(entry, ctx.loops.len() - 1);
}
