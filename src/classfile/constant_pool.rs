use std::io::prelude::*;
use std::io::Result;
use byteorder::{BigEndian, ReadBytesExt};
pub use super::parser::*;

#[derive(Debug)]
pub struct ConstantPool {
    constants: Vec<ConstantInfo>,
}

#[derive(Debug)]
pub enum ConstantInfo {
    Utf8(String),
    Integer(u32),
    Class { name_index: u16 },
    String { string_index: u16 },
    FieldRef {
        class_index: u16,
        name_and_type_index: u16,
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

pub fn parse_constant_pool<R: Read>(input: &mut R) -> Result<Vec<ConstantInfo>> {
    let count = input.read_u16::<BigEndian>()?;
    let mut constant_pool = vec![];
    for _ in 1..count {
        let tag = input.read_u8()?;
        let constant_pool_info = match tag {
            1 => {
                let length = input.read_u16::<BigEndian>()?;
                let mut bytes = vec![0; length as usize];
                input.read_exact(&mut bytes)?;
                let utf8 = String::from_utf8(bytes).unwrap();
                ConstantInfo::Utf8(utf8)
            }
            3 => ConstantInfo::Integer(input.read_u32::<BigEndian>()?),
            7 => ConstantInfo::Class { name_index: input.read_u16::<BigEndian>()? },
            8 => ConstantInfo::String { string_index: input.read_u16::<BigEndian>()? },
            9 => {
                let class_index = input.read_u16::<BigEndian>()?;
                let name_and_type_index = input.read_u16::<BigEndian>()?;
                ConstantInfo::FieldRef {
                    class_index: class_index,
                    name_and_type_index: name_and_type_index,
                }
            }
            10 => {
                let class_index = input.read_u16::<BigEndian>()?;
                let name_and_type_index = input.read_u16::<BigEndian>()?;
                ConstantInfo::MethodRef {
                    class_index: class_index,
                    name_and_type_index: name_and_type_index,
                }
            }
            12 => {
                let name_index = input.read_u16::<BigEndian>()?;
                let descriptor_index = input.read_u16::<BigEndian>()?;
                ConstantInfo::NameAndType {
                    name_index: name_index,
                    descriptor_index: descriptor_index,
                }
            }
            _ => panic!("Unimplemented constant pool info tag: {}", tag),
        };
        constant_pool.push(constant_pool_info);
    }
    Ok(constant_pool)
}

impl ConstantPool {
    pub fn lookup(&self, index: u16) -> &ConstantInfo {
        &self.constants[index as usize - 1]
    }

    pub fn lookup_string(&self, index: u16) -> &str {
        match *self.lookup(index) {
            ConstantInfo::Utf8(ref s) => s,
            ref constant_info => {
                panic!("Error: Expected a UTF8 string looking up {} but found: {:?}",
                       index,
                       constant_info)
            }
        }
    }
}
