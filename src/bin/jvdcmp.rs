extern crate clap;
extern crate jvdcmp;

use jvdcmp::classfile::parser::*;
use jvdcmp::disassembler::transform::*;

fn main() {
    use std::fs::File;
    let matches = clap::App::new("jvdcmp")
        .about("Decompiles Java .class files")
        .arg(clap::Arg::with_name("INPUT")
            .help("Sets the input class file to be decompiled")
            .required(true))
        .get_matches();
    let input = matches.value_of("INPUT").unwrap();
    let mut f = File::open(input).unwrap();
    let class_file = parse_class_file(&mut f).unwrap();
    let compilation_unit = transform(class_file);
    println!("{:#?}", compilation_unit);
}
