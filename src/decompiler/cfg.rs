pub use petgraph::*;
pub use petgraph::graph::*;
pub use petgraph::visit::*;
use disassembler::types::*;
use disassembler::instructions::*;
use pretty::*;

type Label = u32;

#[derive(Debug)]
pub struct Cfg<Stmt, Cond> {
    pub graph: Graph<BasicBlock<Stmt, Cond>, bool, Directed, Label>,
    pub entry_point: usize,
}

impl<Ctx, Stmt, Cond> PrettyWith<Ctx> for Cfg<Stmt, Cond>
    where Stmt: PrettyWith<Ctx>,
          Cond: PrettyWith<Ctx>
{
    fn pretty_with(&self, context: &Ctx) -> Doc {
        let header = doc(format!("start at #{}", self.entry_point)) + newline() + newline();
        let block_docs = self.graph.node_references().map(|node_ref| {
            let node_id = node_ref.id();
            let header = doc(format!("#{}:", node_id.index()));
            let content = node_ref.weight().pretty_with(context);
            let mut edge_refs =
                self.graph.edges_directed(node_id, Direction::Outgoing).collect::<Vec<_>>();
            edge_refs.sort_by_key(|&e| e.weight());
            let gotos = if edge_refs.is_empty() {
                empty()
            } else if edge_refs.len() == 1 {
                let edge_ref = edge_refs[0];
                newline() + format!("goto #{}", edge_ref.target().index())
            } else {
                let gotos = edge_refs.iter().map(|edge_ref| {
                                                     doc(format!("{} => goto #{}",
                                                                 edge_ref.weight(),
                                                                 edge_ref.target().index()))
                                                 });
                let gotos = intersperse(gotos, newline());
                nest(4, newline() + gotos)
            };
            header + newline() + content + gotos
        });
        header + intersperse(block_docs, newline() + newline())
    }
}

#[derive(Clone, Debug)]
pub struct BasicBlock<Stmt, Cond> {
    pub stmts: Vec<Stmt>,
    pub terminator: Option<Cond>,
}

impl<Stmt, Cond> Default for BasicBlock<Stmt, Cond> {
    fn default() -> BasicBlock<Stmt, Cond> {
        BasicBlock {
            stmts: vec![],
            terminator: None,
        }
    }
}

impl<Ctx, Stmt, Cond> PrettyWith<Ctx> for BasicBlock<Stmt, Cond>
    where Stmt: PrettyWith<Ctx>,
          Cond: PrettyWith<Ctx>
{
    fn pretty_with(&self, context: &Ctx) -> Doc {
        let stmts = self.stmts.iter().map(|stmt| stmt.pretty_with(context));
        let terminator = if let Some(ref condition) = self.terminator {
            newline() + nest(4, doc("if (") + condition.pretty_with(context) + ")")
        } else {
            empty()
        };
        intersperse(stmts, newline()) + terminator
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
            Instruction::Return(_) => {
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
            Instruction::Return(_) => {}
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

    // Create empty function entry block:
    bbs.push(BasicBlock::default());
    let entry_point = bbs.len() - 1;
    edges.push((entry_point, 0, false));

    let mut cfg = Cfg {
        graph: Graph::new(),
        entry_point: entry_point,
    };
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
