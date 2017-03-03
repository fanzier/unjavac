#[macro_use]
extern crate bitflags;
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
        constant_pool: constant_pool,
        access_flags: access_flags,
        this_class: this_class,
        super_class: super_class,
        interfaces: interfaces,
        fields: fields,
        methods: methods,
    })
}

fn parse_constant_pool<R: Read>(input: &mut R) -> Result<Vec<ConstantPoolInfo>> {
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
                ConstantPoolInfo::Utf8(utf8)
            }
            3 => ConstantPoolInfo::Integer(input.read_u32::<BigEndian>()?),
            7 => ConstantPoolInfo::Class { name_index: input.read_u16::<BigEndian>()? },
            10 => {
                let class_index = input.read_u16::<BigEndian>()?;
                let name_and_type_index = input.read_u16::<BigEndian>()?;
                ConstantPoolInfo::MethodRef {
                    class_index: class_index,
                    name_and_type_index: name_and_type_index,
                }
            }
            12 => {
                let name_index = input.read_u16::<BigEndian>()?;
                let descriptor_index = input.read_u16::<BigEndian>()?;
                ConstantPoolInfo::NameAndType {
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
struct ClassFile {
    magic: u32,
    minor_version: u16,
    major_version: u16,
    constant_pool: Vec<ConstantPoolInfo>,
    access_flags: AccessFlags,
    this_class: u16,
    super_class: u16,
    interfaces: Vec<u16>,
    fields: Vec<FieldInfo>,
    methods: Vec<MethodInfo>,
}

#[derive(Debug)]
enum ConstantPoolInfo {
    Utf8(String),
    Integer(u32),
    Class { name_index: u16 },
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
struct FieldInfo {
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    attributes: Vec<AttributeInfo>,
}

fn parse_methods<R: Read>(input: &mut R) -> Result<Vec<MethodInfo>> {
    let count = input.read_u16::<BigEndian>()?;
    let mut methods = vec![];
    for _ in 0..count {
        let access_flags = input.read_u16::<BigEndian>()?;
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
struct MethodInfo {
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    attributes: Vec<AttributeInfo>,
}

fn parse_attributes<R: Read>(input: &mut R) -> Result<Vec<AttributeInfo>> {
    let count = input.read_u16::<BigEndian>()?;
    let mut attributes = vec![];
    for _ in 0..count {
        let attribute_name_index = input.read_u16::<BigEndian>()?;
        let attribute_length = input.read_u32::<BigEndian>()?;
        let mut info = vec![0; attribute_length as usize];
        input.read_exact(&mut info);
        let attribute = AttributeInfo {
            attribute_name_index: attribute_name_index,
            info: info,
        };
        attributes.push(attribute);
    }
    Ok(attributes)
}

#[derive(Debug)]
struct AttributeInfo {
    attribute_name_index: u16,
    info: Vec<u8>,
}

bitflags! {
    flags AccessFlags: u16 {
        const ACC_PUBLIC = 0x0001,
        const ACC_FINAL = 0x0010,
        const ACC_SUPER = 0x0020,
        const ACC_INTERFACE = 0x0200,
        const ACC_ABSTRACT = 0x0400,
        const ACC_SYNTHETIC = 0x1000,
        const ACC_ANNOTATION = 0x2000,
        const ACC_ENUM = 0x4000,
    }
}
