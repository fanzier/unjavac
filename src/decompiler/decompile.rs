pub use disassembler::compilation_unit::*;
pub use super::cfg::*;
pub use decompiler::passes::*;

pub fn decompile(unit: CompilationUnit<Code>) {
    for declaration in unit.declarations {
        match declaration {
            Declaration::Method(Method { modifiers: _modifiers,
                                         name,
                                         signature,
                                         code: Some(code) }) => {
                println!("{}: {}:", name, signature);
                let cfg = build_cfg(code);
                println!("{}", cfg);
            }
            _ => unimplemented!(),
        }
    }
}
