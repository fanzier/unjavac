pub use disassembler::compilation_unit::*;
pub use super::cfg::*;
pub use decompiler::passes::*;
pub use decompiler::types::*;

pub fn decompile(unit: CompilationUnit<Code>,
                 verbose: bool)
                 -> CompilationUnit<Cfg<Statement, RecExpr>> {
    let unit = unit.map(|c, _| build_cfg(c));
    if verbose {
        println!(r#"
PASS 1: CONTROL FLOW GRAPH:
===========================
{}"#,
                 unit);
    }
    let unit = stack_to_var::stack_to_vars(unit);
    if verbose {
        println!(r#"
PASS 2: STACK TO VARIABLES:
===========================
{}"#,
                 unit);
    }
    unit
}
