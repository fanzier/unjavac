//! Inlines a variable assignment if it's only used once

use decompiler::cfg::*;
use decompiler::types::*;
use disassembler::types::*;
use std::collections::{HashMap, HashSet};

pub fn var_prop(
    unit: CompilationUnit<Cfg<Statement, Expr>>,
) -> CompilationUnit<Cfg<Statement, Expr>> {
    unit.map(propagate)
}

/// basic block and statement index
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
struct Location(usize, usize);

#[derive(Clone, Debug)]
struct Definition {
    id: Location,
    ident: Ident,
    value: Expr,
    relevant_defs: HashMap<Ident, HashSet<Location>>,
    uses: usize,
    non_propagatable_uses: usize,
}

#[derive(Debug)]
struct DefinitionInfo {
    definitions: HashMap<Location, Definition>,
    relevant_on_bb_entry: Vec<HashMap<Ident, HashSet<Location>>>,
}

fn propagate(mut cfg: Cfg<Statement, Expr>, metadata: &Metadata) -> Cfg<Statement, Expr> {
    let info = collect_def_info(&mut cfg, &metadata);
    println!("{:#?}", info.definitions);
    let mut propagatable_definitions = info
        .definitions
        .into_iter()
        .filter(|(_, d)| is_propagatable(&d))
        .collect::<HashMap<_, _>>();
    propagate_in_definitions(&mut propagatable_definitions);
    propagate_in_code(
        &mut cfg,
        &propagatable_definitions,
        &info.relevant_on_bb_entry,
    );
    cfg
}

fn propagate_in_definitions(definitions: &mut HashMap<Location, Definition>) {
    let ids = definitions.keys().cloned().collect::<Vec<_>>();
    for replace_id in &ids {
        let replace_def = definitions[replace_id].clone();
        println!("ITERATION replacing {:?}:", replace_def);
        for def in definitions.values() {
            println!(
                "{}({},{}) -> {:?}",
                def.ident, def.id.0, def.id.1, def.value
            );
        }
        println!("");
        for def_id in &ids {
            if def_id == replace_id {
                continue; // TODO: is this necessary?
            }
            let replace_def = replace_def.clone();
            let def = &mut definitions.get_mut(def_id).unwrap();
            PropagationVisitor {
                defs: &Some((replace_def.id, replace_def)).into_iter().collect(),
                relevant: &def.relevant_defs,
            }.visit_expr(&mut def.value);
        }
    }
}

fn propagate_in_code(
    cfg: &mut Cfg<Statement, Expr>,
    definitions: &HashMap<Location, Definition>,
    relevant_on_bb_entry: &[HashMap<Ident, HashSet<Location>>],
) {
    for v in cfg.graph.node_indices() {
        let bb_index = v.index();
        let mut relevant = relevant_on_bb_entry[bb_index].clone();
        {
            let bb = cfg.graph.node_weight_mut(v).unwrap();
            for (stmt_idx, stmt) in bb.stmts.iter_mut().enumerate() {
                PropagationVisitor {
                    defs: &definitions,
                    relevant: &relevant,
                }.visit_statement(stmt);
                match stmt {
                    Statement::Expr(Expr::Assign { to, op, from: _ }) => {
                        assert!(op.is_none());
                        match **to {
                            Assignable::Variable(ref var, _) => {
                                let def_id = Location(bb_index, stmt_idx);
                                relevant.insert(var.to_owned(), Some(def_id).into_iter().collect());
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                }
                if definitions.contains_key(&Location(bb_index, stmt_idx)) {
                    *stmt = Statement::Nop;
                }
            }
            if let Some(ref mut cond) = bb.terminator {
                PropagationVisitor {
                    defs: &definitions,
                    relevant: &relevant,
                }.visit_expr(cond);
            }
        }
    }
}

fn collect_def_info(cfg: &mut Cfg<Statement, Expr>, _: &Metadata) -> DefinitionInfo {
    let mut info = DefinitionInfo {
        definitions: HashMap::new(),
        relevant_on_bb_entry: vec![HashMap::new(); cfg.graph.node_count()],
    };
    let mut changed;
    loop {
        changed = false;
        for v in info.definitions.values_mut() {
            v.non_propagatable_uses = 0;
            v.uses = 0;
        }
        for v in cfg.graph.node_indices() {
            let bb_index = v.index();
            let mut relevant = info.relevant_on_bb_entry[bb_index].clone();
            {
                let bb = cfg.graph.node_weight_mut(v).unwrap();
                for (stmt_idx, stmt) in bb.stmts.iter_mut().enumerate() {
                    update_usages(stmt, &relevant, &mut info.definitions);
                    match stmt {
                        Statement::Expr(Expr::Assign { to, op, from }) => {
                            assert!(op.is_none());
                            match **to {
                                Assignable::Variable(ref var, _) => {
                                    let def_id = Location(bb_index, stmt_idx);
                                    if !info.definitions.contains_key(&def_id) {
                                        info.definitions.insert(
                                            def_id,
                                            Definition {
                                                id: def_id,
                                                ident: var.to_owned(),
                                                value: (**from).clone(),
                                                relevant_defs: relevant.clone(),
                                                uses: 0,
                                                non_propagatable_uses: 0,
                                            },
                                        );
                                        changed = true;
                                    }
                                    info.definitions.get_mut(&def_id).unwrap().relevant_defs =
                                        relevant.clone();
                                    relevant
                                        .insert(var.to_owned(), Some(def_id).into_iter().collect());
                                }
                                _ => (),
                            }
                        }
                        _ => (),
                    }
                }
            }
            for target_node in cfg.graph.neighbors(v) {
                let target_index = target_node.index();
                let target_relevant = &mut info.relevant_on_bb_entry[target_index];
                for (var, def_ids) in relevant.iter() {
                    let before = target_relevant.entry(var.to_owned()).or_insert_with(|| {
                        changed = true;
                        def_ids.clone()
                    });
                    if !def_ids.is_subset(before) {
                        changed = true;
                        before.extend(def_ids);
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    info
}

fn update_usages(
    stmt: &mut Statement,
    relevant_defs: &HashMap<Ident, HashSet<Location>>,
    definitions: &mut HashMap<Location, Definition>,
) {
    struct UsageVisitor<'a>(
        &'a mut HashMap<Location, Definition>,
        &'a HashMap<Ident, HashSet<Location>>,
    );
    impl<'a> Visitor for UsageVisitor<'a> {
        fn visit_expr(&mut self, expr: &mut Expr) {
            match *expr {
                Expr::Assignable(ref assignable) => match **assignable {
                    Assignable::Variable(ref var, _) => {
                        let empty = HashSet::new();
                        let possible_definitions = self.1.get(var).unwrap_or(&empty);
                        let propagatable = possible_definitions.len() <= 1;
                        for def_id in possible_definitions {
                            let def = self.0.get_mut(def_id).unwrap();
                            def.uses += 1;
                            if !propagatable {
                                def.non_propagatable_uses += 1;
                            }
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
            walk_expr(self, expr);
        }
    }
    UsageVisitor(definitions, relevant_defs).visit_statement(stmt);
}

fn is_propagatable(def: &Definition) -> bool {
    def.non_propagatable_uses == 0 && def.uses <= 1
}

struct PropagationVisitor<'a> {
    defs: &'a HashMap<Location, Definition>,
    relevant: &'a HashMap<Ident, HashSet<Location>>,
}
impl<'a> Visitor for PropagationVisitor<'a> {
    fn visit_expr(&mut self, expr: &mut Expr) {
        let replace = match *expr {
            Expr::Assignable(ref assignable) => match **assignable {
                Assignable::Variable(ref var, _) if self.relevant.contains_key(var) => {
                    let possible_definitions = &self.relevant[var];
                    if possible_definitions.len() == 1 {
                        let def_id = possible_definitions.iter().next().unwrap();
                        if let Some(def) = self.defs.get(def_id) {
                            Some(def.value.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        };
        if let Some(replace) = replace {
            *expr = replace;
        } else {
            walk_expr(self, expr);
        }
    }
}
