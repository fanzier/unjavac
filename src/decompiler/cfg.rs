pub use petgraph::*;
pub use petgraph::graph::*;
pub use petgraph::visit::*;
use super::super::disassembler::compilation_unit::*;
use super::super::disassembler::instructions::*;
use std::fmt::{Display, Formatter, Result};

type Label = u32;

#[derive(Debug)]
pub struct Cfg<Stmt, Cond> {
    pub graph: Graph<BasicBlock<Stmt, Cond>, bool, Directed, Label>,
}

impl<Stmt: Display, Cond: Display> Display for Cfg<Stmt, Cond> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        for node_ref in self.graph.node_references() {
            let node_id = node_ref.id();
            writeln!(f, "#{}:", node_id.index())?;
            write!(f, "{}", node_ref.weight())?;
            let mut edge_refs =
                self.graph.edges_directed(node_id, Direction::Outgoing).collect::<Vec<_>>();
            edge_refs.sort_by_key(|&e| e.weight());
            if edge_refs.len() == 1 {
                let edge_ref = edge_refs[0];
                writeln!(f, "goto #{}", edge_ref.target().index())?;
            } else {
                for edge_ref in edge_refs {
                    writeln!(f,
                             "  {} => goto #{}",
                             edge_ref.weight(),
                             edge_ref.target().index())?;
                }
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct BasicBlock<Stmt, Cond> {
    pub stmts: Vec<Stmt>,
    pub terminator: Option<Cond>,
}

impl<Stmt: Display, Cond: Display> Display for BasicBlock<Stmt, Cond> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        for stmt in &self.stmts {
            writeln!(f, "{}", stmt)?;
        }
        if let Some(ref condition) = self.terminator {
            writeln!(f, "if {}", condition)?;
        }
        Ok(())
    }
}

pub fn build_cfg(code: Code) -> Cfg<Instruction, JumpCondition> {
    use std::collections::HashSet;
    use std::collections::HashMap;
    let instrs = code.instructions;

    let mut index_to_pc = HashMap::new();
    let mut pc_to_index = HashMap::new();
    for (i, &(pc, _)) in instrs.iter().enumerate() {
        index_to_pc.insert(i, pc);
        pc_to_index.insert(pc, i);
    }

    let mut bb_starts = HashSet::new();
    bb_starts.insert(0);
    for &(pc, ref instr) in &instrs {
        let next_pc = pc_to_index[&pc] + 1;
        let next_pc = if next_pc < instrs.len() {
            Some(next_pc)
        } else {
            None
        };
        match *instr {
            Instruction::Jump(Jump { address, .. }) => {
                // next instruction starts a block:
                if let Some(next_pc) = next_pc {
                    bb_starts.insert(next_pc);
                }
                // branch address starts a block:
                bb_starts.insert(pc_to_index[&address]);
            }
            Instruction::Return => {
                // next instruction starts a block:
                if let Some(next_pc) = next_pc {
                    bb_starts.insert(next_pc);
                }
            }
            _ => (),
        }
    }
    let mut bb_starts = bb_starts.iter().cloned().collect::<Vec<_>>();
    bb_starts.sort();

    bb_starts.reverse(); // start at the end
    let mut rest = instrs;
    let mut blocks = vec![];
    for &start_index in &bb_starts {
        let block = rest.split_off(start_index);
        blocks.push(block);
    }
    blocks.reverse();
    bb_starts.reverse();

    let mut pc_to_bb_id = HashMap::new();
    for (i, &start_index) in bb_starts.iter().enumerate() {
        let start_pc = index_to_pc[&start_index];
        pc_to_bb_id.insert(start_pc, i);
    }

    let mut bbs = vec![];
    let mut edges = vec![];
    for (block_id, mut block) in blocks.drain(..).enumerate() {
        let mut terminator = None;
        let mut delete_last = false;
        match block.last().unwrap().1 {
            Instruction::Jump(Jump { condition, address }) => {
                if condition.is_some() {
                    edges.push((block_id, block_id + 1, false));
                    terminator = condition;
                }
                edges.push((block_id, pc_to_bb_id[&address], true));
                delete_last = true;
            }
            Instruction::Return => {}
            _ => {
                edges.push((block_id, block_id + 1, false));
            }
        }
        if delete_last {
            block.pop().unwrap();
        }
        let block = block.drain(..).map(|(_, instr)| instr).collect::<Vec<_>>();
        let bb = BasicBlock {
            stmts: block,
            terminator: terminator,
        };
        bbs.push(bb);
    }

    let mut cfg = Cfg { graph: Graph::new() };
    for bb in bbs {
        cfg.graph.add_node(bb);
    }
    for (from, to, edge) in edges {
        cfg.graph.add_edge(NodeIndex::from(from as Label),
                           NodeIndex::from(to as Label),
                           edge);
    }
    cfg
}
