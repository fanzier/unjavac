use disassembler::instructions::*;
use disassembler::types::*;
pub use petgraph::graph::*;
pub use petgraph::visit::*;
pub use petgraph::*;
use pretty::*;

use std::collections::BTreeMap;

pub type Label = NodeIndex<LabelIndex>;
pub type LabelIndex = usize;
pub type CfgGraph<Stmt, Cond> = Graph<BasicBlock<Stmt, Cond>, bool, Directed, LabelIndex>;
pub type Edge = bool;
type Map<K, T> = BTreeMap<K, T>;

#[derive(Debug, Clone)]
pub struct Cfg<Stmt, Cond> {
    pub graph: CfgGraph<Stmt, Cond>,
    pub entry_point: Label,
    pub exit_point: Label,
}

impl<Stmt, Cond> Cfg<Stmt, Cond> {
    pub fn map<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Vec<Stmt>),
    {
        self.graph
            .node_weights_mut()
            .for_each(|node| f(&mut node.stmts));
    }
}

impl<Ctx, Stmt, Cond> PrettyWith<Ctx> for Cfg<Stmt, Cond>
where
    Stmt: PrettyWith<Ctx>,
    Cond: PrettyWith<Ctx>,
{
    fn pretty_with(&self, context: &Ctx) -> Doc {
        let header = doc(format!("start at #{}", self.entry_point.index())) + newline() + newline();
        let block_docs = self.graph.node_references().map(|node_ref| {
            let node_id = node_ref.id();
            let header = doc(format!("#{}:", node_id.index()));
            let content = node_ref.weight().pretty_with(context);
            let mut edge_refs = self.graph
                .edges_directed(node_id, Direction::Outgoing)
                .collect::<Vec<_>>();
            edge_refs.sort_by_key(|&e| e.weight());
            let gotos = if edge_refs.is_empty() {
                empty()
            } else if edge_refs.len() == 1 {
                let edge_ref = edge_refs[0];
                newline() + format!("goto #{}", edge_ref.target().index())
            } else {
                let gotos = edge_refs.iter().map(|edge_ref| {
                    doc(format!(
                        "{} => goto #{}",
                        edge_ref.weight(),
                        edge_ref.target().index()
                    ))
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
pub struct Dominators {
    root: Label,
    map: Map<Label, Label>,
    reversed: bool,
}

impl Dominators {
    pub fn root(&self) -> Label {
        self.root
    }

    pub fn is_for(&self, node: Label, mut check: Label) -> bool {
        if check == node {
            return true;
        }
        while let Some(imm) = self.get_immediate(check) {
            if imm == node {
                return true;
            }
            check = imm;
        }
        false
    }

    pub fn get_immediate(&self, node: Label) -> Option<Label> {
        self.map.get(&node).cloned()
    }

    pub fn get_all(&self, mut node: Label) -> Vec<Label> {
        let mut path = vec![node];
        while let Some(imm) = self.get_immediate(node) {
            node = imm;
            path.push(node);
        }
        path
    }

    // TODO use iterators?
    pub fn get_common(&self, nodes: &[Label]) -> Option<Label> {
        if nodes.is_empty() {
            return None;
        }
        let mut paths = vec![];
        for &node in nodes {
            let mut path = self.get_all(node);
            path.reverse();
            paths.push(path);
        }
        let mut nearest = None;
        for distance in 0..paths[0].len() {
            let node = paths[0][distance];
            for path in &paths {
                if path.get(distance) != Some(&node) {
                    return nearest;
                }
            }
            nearest = Some(node);
        }
        nearest
    }

    fn pretty_from(&self, root: Label, reverse_map: &Map<Label, Vec<Label>>) -> Doc {
        let children = if let Some(children) = reverse_map.get(&root) {
            children
        } else {
            return newline() + doc(root.index());
        };
        let subtree = intersperse(
            children.iter().map(|&l| self.pretty_from(l, reverse_map)),
            empty(),
        );
        if self.reversed {
            nest(2, subtree) + newline() + doc(root.index())
        } else {
            newline() + doc(root.index()) + nest(2, subtree)
        }
    }
}

impl<T> PrettyWith<T> for Dominators {
    fn pretty_with(&self, _: &T) -> Doc {
        let mut reverse_map = Map::new();
        for (&node, &parent) in &self.map {
            reverse_map
                .entry(parent)
                .or_insert_with(Vec::new)
                .push(node);
        }
        doc("(Post)Dominators:") + self.pretty_from(self.root(), &reverse_map)
    }
}

impl<Stmt, Cond> Cfg<Stmt, Cond> {
    pub fn compute_dominators(&self, post: bool) -> Dominators {
        let mut map = Map::new();
        let dominators = if post {
            algo::dominators::simple_fast(visit::Reversed(&self.graph), self.exit_point)
        } else {
            algo::dominators::simple_fast(&self.graph, self.entry_point)
        };
        for node in self.graph.node_indices() {
            if let Some(dom) = dominators.immediate_dominator(node) {
                map.insert(node, dom);
            }
        }
        let root = dominators.root();
        Dominators {
            root: root,
            map: map,
            reversed: post,
        }
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
where
    Stmt: PrettyWith<Ctx>,
    Cond: PrettyWith<Ctx>,
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
    use std::collections::HashMap;
    use std::collections::HashSet;
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
    let entry_point = entry_point.into();

    // Create empty function exit block:
    bbs.push(BasicBlock::default());
    let exit_point = (bbs.len() - 1).into();

    let mut cfg = Cfg {
        graph: Graph::with_capacity(bbs.len(), edges.len()),
        entry_point: entry_point,
        exit_point: exit_point,
    };
    for bb in bbs {
        cfg.graph.add_node(bb);
    }
    for (from, to, edge) in edges {
        cfg.graph.add_edge(from.into(), to.into(), edge);
    }
    let mut outdegree_zero_nodes = vec![];
    for node in cfg.graph.node_indices() {
        let outdegree_zero = {
            let mut outgoing = cfg.graph.neighbors_directed(node, Direction::Outgoing);
            outgoing.next().is_none()
        };
        if outdegree_zero && node != exit_point {
            outdegree_zero_nodes.push(node);
        }
    }
    for node in outdegree_zero_nodes {
        cfg.graph.add_edge(node, exit_point, false);
    }
    cfg
}
