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
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Jump(Label, Label);

pub fn structure(unit: CompilationUnit<Cfg<Statement, RecExpr>>) -> CompilationUnit<Block> {
    unit.map(structure_cfg)
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct Loop {
    nodes: Set<Label>,
    entry: Label,
    continue_edges: Set<Jump>,
    exits: Set<Label>,
    break_point: Label,
    break_edges: Set<Jump>,
}

fn loop_label(index: usize) -> String {
    format!("loop_{}", index)
}

#[derive(Debug)]
struct Context {
    cfg: Cfg<Statement, RecExpr>,
    loops: Vec<Loop>,
    entry_to_loop_index: Map<Label, usize>,
    loop_breaks: Map<Jump, usize>, // jump -> loop index
    dominators: Dominators,
    postdominators: Dominators,
}

fn create_context(cfg: Cfg<Statement, RecExpr>) -> Context {
    let dominators = cfg.compute_dominators(false);
    let postdominators = cfg.compute_dominators(true);
    Context {
        cfg: cfg,
        loops: vec![],
        entry_to_loop_index: Map::new(),
        loop_breaks: Map::new(),
        dominators: dominators,
        postdominators: postdominators,
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
    let exit = ctx.cfg.exit_point;
    let result = structure_from_to(&mut ctx, entry, exit);
    Block(vec![], result)
}

fn structure_from_to(ctx: &mut Context, mut cur: Label, stop: Label) -> Vec<Statement> {
    let mut result = vec![];
    while cur != stop && cur != ctx.cfg.exit_point {
        cur = translate_block(ctx, &mut result, cur, stop);
    }
    result
}

fn handle_jump(ctx: &mut Context, result: &mut Vec<Statement>, jump: Jump, stop: Label) -> Label {
    let Jump(_cur, next) = jump;
    assert!(ctx.postdominators.is_for(stop, next),
            "stop point {} doesn't postdominate next {}",
            stop.index(),
            next.index());

    if let Some(&loop_index) = ctx.entry_to_loop_index.get(&next) {
        // It's a jump to a loop entry
        let label = loop_label(loop_index);
        if ctx.loops[loop_index].continue_edges.contains(&jump) {
            result.push(Statement::Continue(Some(label)));
            return stop;
        } else {
            let brk = ctx.loops[loop_index].break_point;
            assert!(ctx.postdominators.is_for(stop, brk),
                    "stop point {} doesn't postdominate brk {}",
                    stop.index(),
                    brk.index());
            let body = structure_from_to(ctx, next, brk);
            result.push(Statement::While {
                            label: Some(label),
                            cond: mk_variable("true".into()),
                            body: Block(vec![], body),
                            do_while: false,
                        });
            return brk;
        }
    }
    if let Some(&loop_index) = ctx.loop_breaks.get(&jump) {
        // It's a jump out of the loop (break)
        let label = loop_label(loop_index);
        result.push(Statement::Break(Some(label)));
        return ctx.loops[loop_index].break_point;
    }
    next
}

fn translate_block(ctx: &mut Context,
                   result: &mut Vec<Statement>,
                   cur: Label,
                   stop: Label)
                   -> Label {
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
        let join = ctx.postdominators.get_immediate(cur).unwrap();
        assert!(ctx.postdominators.is_for(stop, join),
                "stop point {} doesn't postdominate the join point {}",
                stop.index(),
                join.index());
        let mut then_stmts = vec![];
        let then_block = handle_jump(ctx, &mut then_stmts, Jump(cur, outgoing[&true]), stop);
        then_stmts.append(&mut structure_from_to(ctx, then_block, join));
        let mut else_stmts = vec![];
        let else_block = handle_jump(ctx, &mut else_stmts, Jump(cur, outgoing[&false]), stop);
        else_stmts.append(&mut structure_from_to(ctx, else_block, join));
        result.push(Statement::If {
                        cond: cond,
                        then: Block(vec![], then_stmts),
                        els: Some(Block(vec![], else_stmts)),
                    });
        join
    } else {
        assert!(outgoing.len() <= 1,
                "basic block #{} has too many successors",
                cur.index());
        let next = *outgoing.values().next().expect(&format!("basic block {} has no outgoing edges",
                                                             cur.index()));
        handle_jump(ctx, result, Jump(cur, next), stop)
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
        let lupe = find_entries_and_exits(ctx, nodes.clone());
        let entry = lupe.entry;
        store_loop_in_context(ctx, lupe);
        // recursively collect the nested loops inside this loop:
        nodes.remove(&entry);
        collect_loops(ctx, &nodes);
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

fn find_entries_and_exits(ctx: &Context, nodes: Set<Label>) -> Loop {
    let mut entry_points = Set::new();
    let mut exit_points = Set::new();
    for &node in &nodes {
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
    // extract the entry point:
    assert!(entry_points.len() <= 1,
            "The loop consisting of the basic blocks {:?} has multiple entry points {:?}",
            nodes,
            entry_points);
    let entry_point = *entry_points.iter().next().unwrap();
    let mut continue_edges = Set::new();
    {
        let graph = &ctx.cfg.graph;
        let incoming_neighbors = graph.neighbors_directed(entry_point, Direction::Incoming);
        for incoming in incoming_neighbors {
            if nodes.contains(&incoming) {
                continue_edges.insert(Jump(incoming, entry_point));
            }
        }
    }
    let break_point = find_best_break_block(ctx, &exit_points);
    let mut break_edges = Set::new();
    {
        let graph = &ctx.cfg.graph;
        let incoming_neighbors = graph.neighbors_directed(break_point, Direction::Incoming);
        for incoming in incoming_neighbors {
            if ctx.dominators.is_for(entry_point, break_point) {
                break_edges.insert(Jump(incoming, break_point));
            }
        }
    }
    Loop {
        nodes: nodes,
        entry: entry_point,
        continue_edges: continue_edges,
        exits: exit_points,
        break_point: break_point,
        break_edges: break_edges,
    }
}

fn find_best_break_block(ctx: &Context, exits: &Set<Label>) -> Label {
    let mut exits = exits.clone();
    // TODO: This can be improved if the CFG looks like this:
    // A ----------------> exit
    // B --> D ----> E -==-^
    // C ----^
    // Here, we should pick D to be the best beak_block, not exit.
    ctx.postdominators.get_common(&exits.iter().cloned().collect::<Vec<_>>()).unwrap()
}

fn store_loop_in_context(ctx: &mut Context, lupe: Loop) {
    let loop_index = ctx.loops.len();
    ctx.entry_to_loop_index.insert(lupe.entry, loop_index);
    for edge in lupe.break_edges.clone() {
        ctx.loop_breaks.insert(edge, loop_index);
    }
    ctx.loops.push(lupe);
}
