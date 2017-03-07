pub use super::super::classfile::parser::*;
pub use super::class::*;

use byteorder::{ReadBytesExt, BigEndian};
use std::io::{Result, Read};

#[derive(Debug)]
pub struct CodeAttribute {
    max_stack: u16,
    max_local: u16,
    code: Vec<u8>,
    exception_table: Vec<ExceptionTableEntry>,
    attributes: Vec<AttributeInfo>,
}

#[derive(Debug)]
pub struct ExceptionTableEntry {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16,
}

pub fn parse_code_attribute(bytes: &[u8]) -> Result<CodeAttribute> {
    use std::io::Cursor;
    let mut input = Cursor::new(bytes);
    let max_stack = input.read_u16::<BigEndian>()?;
    let max_local = input.read_u16::<BigEndian>()?;
    let code_length = input.read_u32::<BigEndian>()?;
    let mut code = vec![0; code_length as usize];
    input.read_exact(&mut code)?;
    let exception_table_length = input.read_u16::<BigEndian>()?;
    let mut exception_table = vec![];
    for _ in 0..exception_table_length {
        let start_pc = input.read_u16::<BigEndian>()?;
        let end_pc = input.read_u16::<BigEndian>()?;
        let handler_pc = input.read_u16::<BigEndian>()?;
        let catch_type = input.read_u16::<BigEndian>()?;
        exception_table.push(ExceptionTableEntry {
            start_pc: start_pc,
            end_pc: end_pc,
            handler_pc: handler_pc,
            catch_type: catch_type,
        });
    }
    let attributes = parse_attributes(&mut input)?;
    Ok(CodeAttribute {
        max_stack: max_stack,
        max_local: max_local,
        code: code,
        exception_table: exception_table,
        attributes: attributes,
    })
}

pub fn disassemble(code: CodeAttribute) -> Code {
    let len = code.code.len();
    let mut instructions = Vec::with_capacity(len);
    let mut bytes = code.code.iter().cloned();
    use std::iter::ExactSizeIterator;
    while let Some(opcode) = bytes.next() {
        let pc = len - bytes.len() - 1;
        let instruction = decode_instruction(opcode, pc as u16, &mut bytes);
        instructions.push((pc as u16, instruction));
    }
    Code { instructions: instructions }
}
