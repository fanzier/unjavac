extern crate byteorder;

use std::io::prelude::*;
use std::io::Result;
use byteorder::{BigEndian, ReadBytesExt};

fn main() {
    use std::fs::File;
    let mut f = File::open("java-bytecode-test/Main.class").unwrap();
    let class_file = parse_class_file(&mut f).unwrap();
    println!("{:?}", class_file);
}

fn parse_class_file<R: Read>(input: &mut R) -> Result<ClassFile> {
    let magic = input.read_u32::<BigEndian>()?;
    let minor_version = input.read_u16::<BigEndian>()?;
    let major_version = input.read_u16::<BigEndian>()?;
    let constant_pool = parse_constant_pool(input)?;
    let access_flags = input.read_u16::<BigEndian>()?;
    let this_class = input.read_u16::<BigEndian>()?;
    let super_class = input.read_u16::<BigEndian>()?;
    Ok(ClassFile {
        magic: magic,
        minor_version : minor_version,
        major_version : major_version,
        constant_pool : constant_pool,
        access_flags : access_flags,
        this_class : this_class,
        super_class : super_class
    })
}

fn parse_constant_pool<R: Read>(input: &mut R) -> Result<Vec<ConstantPoolInfo>> {
    let count = input.read_u16::<BigEndian>()?;
    let mut constant_pool = vec![];
    for i in 1..count {
        let tag = input.read_u8()?;
        let constant_pool_info = match tag {
            1 => {
                let length = input.read_u16::<BigEndian>()?;
                let mut bytes = vec![0; length as usize];
                input.read_exact(&mut bytes)?;
                let utf8 = String::from_utf8(bytes).unwrap();
                ConstantPoolInfo::Utf8(utf8)
            },
            3 => ConstantPoolInfo::Integer(input.read_u32::<BigEndian>()?),
            7 => ConstantPoolInfo::Class{ name_index: input.read_u16::<BigEndian>()? },
            10 => {
                let class_index = input.read_u16::<BigEndian>()?;
                let name_and_type_index = input.read_u16::<BigEndian>()?;
                ConstantPoolInfo::MethodRef {
                    class_index: class_index,
                    name_and_type_index: name_and_type_index
                }
            },
            12 => {
                let name_index = input.read_u16::<BigEndian>()?;
                let descriptor_index = input.read_u16::<BigEndian>()?;
                ConstantPoolInfo::NameAndType {
                    name_index: name_index,
                    descriptor_index: descriptor_index,
                }
            },
            _ => panic!("Unimplemented constant pool info tag: {}", tag),
        };
        constant_pool.push(constant_pool_info);
    }
    Ok(constant_pool)
}

#[derive(Debug)]
struct ClassFile {
    magic : u32,
    minor_version : u16,
    major_version : u16,
    constant_pool : Vec<ConstantPoolInfo>,
    access_flags : u16,
    this_class : u16,
    super_class : u16, /*
    interfaces : Vec<interface_info>,
    fields : Vec<fields_info>,
    methods : Vec<method_info>,
    attributes : Vec<attribute_info>, */
}

#[derive(Debug)]
enum ConstantPoolInfo {
    Utf8(String),
    Integer(u32),
    Class {
        name_index: u16,
    },
    MethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
}
