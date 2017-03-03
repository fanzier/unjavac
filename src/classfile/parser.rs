use std::io::prelude::*;
use std::io::Result;
use byteorder::{BigEndian, ReadBytesExt};
pub use super::constants::*;

pub fn parse_class_file<R: Read>(input: &mut R) -> Result<ClassFile> {
    let magic = input.read_u32::<BigEndian>()?;
    let minor_version = input.read_u16::<BigEndian>()?;
    let major_version = input.read_u16::<BigEndian>()?;
    let constant_pool = parse_constant_pool(input)?;
    let access_flags = AccessFlags::from_bits(input.read_u16::<BigEndian>()?).unwrap();
    let this_class = input.read_u16::<BigEndian>()?;
    let super_class = input.read_u16::<BigEndian>()?;
    let interfaces = parse_interfaces(input)?;
    let fields = parse_fields(input)?;
    let methods = parse_methods(input)?;
    Ok(ClassFile {
        magic: magic,
        minor_version: minor_version,
        major_version: major_version,
        constant_pool: ConstantPool { constants: constant_pool },
        access_flags: access_flags,
        this_class: this_class,
        super_class: super_class,
        interfaces: interfaces,
        fields: fields,
        methods: methods,
    })
}

fn parse_constant_pool<R: Read>(input: &mut R) -> Result<Vec<ConstantInfo>> {
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

#[derive(Debug)]
pub struct ClassFile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: ConstantPool,
    pub access_flags: AccessFlags,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
}

#[derive(Debug)]
pub struct ConstantPool {
    constants: Vec<ConstantInfo>,
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

#[derive(Debug)]
pub enum ConstantInfo {
    Utf8(String),
    Integer(u32),
    Class { name_index: u16 },
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

fn parse_interfaces<R: Read>(input: &mut R) -> Result<Vec<u16>> {
    let count = input.read_u16::<BigEndian>()?;
    let mut interfaces = vec![];
    for _ in 0..count {
        unimplemented!()
    }
    Ok(interfaces)
}

fn parse_fields<R: Read>(input: &mut R) -> Result<Vec<FieldInfo>> {
    let count = input.read_u16::<BigEndian>()?;
    let mut fields = vec![];
    for _ in 0..count {
        unimplemented!()
    }
    Ok(fields)
}

#[derive(Debug)]
pub struct FieldInfo {
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    attributes: Vec<AttributeInfo>,
}

fn parse_methods<R: Read>(input: &mut R) -> Result<Vec<MethodInfo>> {
    let count = input.read_u16::<BigEndian>()?;
    let mut methods = vec![];
    for _ in 0..count {
        let access_flags = AccessFlags::from_bits(input.read_u16::<BigEndian>()?).unwrap();
        let name_index = input.read_u16::<BigEndian>()?;
        let descriptor_index = input.read_u16::<BigEndian>()?;
        let attributes = parse_attributes(input)?;
        let method = MethodInfo {
            access_flags: access_flags,
            name_index: name_index,
            descriptor_index: descriptor_index,
            attributes: attributes,
        };
        methods.push(method);
    }
    Ok(methods)
}

#[derive(Debug)]
pub struct MethodInfo {
    pub access_flags: AccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>,
}

fn parse_attributes<R: Read>(input: &mut R) -> Result<Vec<AttributeInfo>> {
    let count = input.read_u16::<BigEndian>()?;
    let mut attributes = vec![];
    for _ in 0..count {
        let attribute_name_index = input.read_u16::<BigEndian>()?;
        let attribute_length = input.read_u32::<BigEndian>()?;
        let mut info = vec![0; attribute_length as usize];
        input.read_exact(&mut info)?;
        let attribute = AttributeInfo {
            name_index: attribute_name_index,
            info: info,
        };
        attributes.push(attribute);
    }
    Ok(attributes)
}

#[derive(Debug)]
pub struct AttributeInfo {
    pub name_index: u16,
    pub info: Vec<u8>,
}
