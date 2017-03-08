pub use disassembler::class::*;
pub use super::cfg::*;

pub fn decompile(unit: CompilationUnit) {
    for declaration in unit.declarations {
        match declaration {
            Declaration::Method { modifiers: _, name, signature, code: Some(code) } => {
                println!("{}: {}:", name, signature);
                let cfg = build_cfg(code);
                println!("{}", cfg);
            }
            _ => unimplemented!(),
        }
    }
}
