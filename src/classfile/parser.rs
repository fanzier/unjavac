use std::io::prelude::*;
use std::io::Result;
use byteorder::{BigEndian, ReadBytesExt};
pub use super::constants::*;
pub use super::constant_pool::*;

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

fn parse_interfaces<R: Read>(input: &mut R) -> Result<Vec<u16>> {
    let count = input.read_u16::<BigEndian>()?;
    let interfaces = vec![];
    for _ in 0..count {
        unimplemented!()
    }
    Ok(interfaces)
}

fn parse_fields<R: Read>(input: &mut R) -> Result<Vec<FieldInfo>> {
    let count = input.read_u16::<BigEndian>()?;
    let fields = vec![];
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

pub fn parse_attributes<R: Read>(input: &mut R) -> Result<Vec<AttributeInfo>> {
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
