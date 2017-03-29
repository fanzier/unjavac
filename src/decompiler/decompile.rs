pub use disassembler::types::*;
pub use decompiler::cfg::*;
pub use decompiler::passes::*;
pub use decompiler::types::*;

pub fn decompile(unit: CompilationUnit<Code>, verbose: bool) -> CompilationUnit<Block> {
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
    let unit = structure::structure(unit);
    if verbose {
        println!(r#"
PASS 3: STRUCTURE THE CONRTOL FLOW GRAPH:
=========================================
{}"#,
                 unit);
    }
    unit
}
