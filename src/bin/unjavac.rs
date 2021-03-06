extern crate clap;
extern crate unjavac;

use unjavac::classfile::parser::*;
use unjavac::decompiler::decompile::*;
use unjavac::disassembler::transform::*;

fn main() {
    use std::fs::File;
    let matches = clap::App::new("unjavac")
        .about("Decompiles Java .class files")
        .arg(
            clap::Arg::with_name("INPUT")
                .help("Sets the input class file to be decompiled")
                .required(true),
        )
        .get_matches();
    let input = matches.value_of("INPUT").unwrap();
    let mut f = File::open(input).unwrap();
    let class_file = parse_class_file(&mut f).unwrap();
    let compilation_unit = transform(&class_file);
    println!(
        r#"
DISASSEMBLY:
============
{:#?}"#,
        compilation_unit
    );
    println!(
        r#"
DISASSEMBLY PRETTY-PRINTED:
===========================
{}"#,
        compilation_unit
    );
    decompile(compilation_unit, true);
}
