extern crate jvdcmp;

use std::io::prelude::*;
use jvdcmp::classfile::parser::*;

fn main() {
    use std::fs::File;
    let mut f = File::open("java-bytecode-test/Main.class").unwrap();
    let class_file = parse_class_file(&mut f).unwrap();
    println!("{:?}", class_file);
}
