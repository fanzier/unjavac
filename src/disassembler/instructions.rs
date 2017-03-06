pub use super::class::*;

#[derive(Copy, Clone, Debug)]
pub enum Kind {
    B, // byte
    S, // short
    C, // char
    I, // int
    L, // long
    F, // float
    D, // double
    A, // reference
}

#[derive(Debug)]
pub enum Instruction {
    Nop,
    Load(Load),
    Store(Store),
    Arithm(Arithm),
    TypeConv(TypeConv),
    ObjManip(ObjManip),
    StackManage(StackManage),
    ControlTransfer(ControlTransfer),
    Invoke(Invoke),
    Throw,
    Return,
    Synchronized(Synchronized),
    Invalid(u8),
}

pub fn decode_instruction<I: Iterator<Item = u8>>(opcode: u8, iter: &mut I) -> Instruction {
    use self::Instruction::*;
    println!("Decoding opcode 0x{:x}.", opcode);
    match opcode {
        0x00 => Nop,
        0x01...0x35 => Load(decode_load(opcode, iter)),
        0x36...0x56 => Store(decode_store(opcode, iter)),
        0x57...0x5f => unimplemented!(), // stack Stack
        0x60...0x84 => unimplemented!(), // arithmetic
        0x85...0x93 => unimplemented!(), // type conversion
        0x94...0x98 => unimplemented!(), // comparison (arithmetic)
        0x99...0xab => unimplemented!(), // control flow
        0xac...0xb1 => Return,
        0xb2...0xb5 => ObjManip(decode_object_manip(opcode, iter)),
        0xb6...0xba => Invoke(decode_invoke(opcode, iter)),
        0xbb...0xbe => unimplemented!(), // object manip
        0xbf => Throw,
        0xc0...0xc1 => unimplemented!(), // object manip
        0xc2...0xc3 => unimplemented!(), // monitor{enter|exit}
        0xc4...0xc9 => unimplemented!(), // miscalleneous
        0xca...0xff => panic!("Invalid opcode 0x{:x}", opcode),
        _ => unreachable!(), // no other possibilities possible but rustc can't see this
    }
}

pub fn read_u16_index<I: Iterator<Item = u8>>(iter: &mut I) -> u16 {
    let index1 = iter.next().unwrap();
    let index2 = iter.next().unwrap();
    (index1 as u16) << 8 | index2 as u16
}

#[derive(Debug)]
pub enum Load {
    Var(Kind, usize),
    Ldc(u16),
}

pub fn decode_load<I: Iterator<Item = u8>>(opcode: u8, iter: &mut I) -> Load {
    use self::Load::*;
    match opcode {
        0x12 => {
            let index = iter.next().unwrap();
            Ldc(index as u16)
        }
        0x1a...0x1d => Var(Kind::I, (opcode - 0x1a) as usize),
        0x2a...0x2d => Var(Kind::A, (opcode - 0x2a) as usize),
        _ => unimplemented!(),
    }
}

#[derive(Debug)]
pub enum Store {
    Var(Kind, usize),
}

pub fn decode_store<I: Iterator<Item = u8>>(opcode: u8, iter: &mut I) -> Store {
    use self::Store::*;
    match opcode {
        0x3b...0x3e => Var(Kind::I, (opcode - 0x3b) as usize),
        _ => unimplemented!(),
    }
}

#[derive(Debug)]
pub enum Arithm {}

#[derive(Debug)]
pub enum TypeConv {}

#[derive(Debug)]
pub enum ObjManip {
    Access(GetOrPut, StaticOrField, u16),
}

#[derive(Copy, Clone, Debug)]
pub enum GetOrPut {
    Get,
    Put,
}

#[derive(Copy, Clone, Debug)]
pub enum StaticOrField {
    Static,
    Field,
}

pub fn decode_object_manip<I: Iterator<Item = u8>>(opcode: u8, iter: &mut I) -> ObjManip {
    use self::ObjManip::*;
    use self::GetOrPut::*;
    use self::StaticOrField::*;
    let index = read_u16_index(iter);
    match opcode {
        0xb2 => Access(Get, Static, index),
        0xb3 => Access(Put, Static, index),
        0xb4 => Access(Get, Field, index),
        0xb5 => Access(Put, Field, index),
        _ => unimplemented!(),
    }
}

#[derive(Debug)]
pub enum StackManage {}

#[derive(Debug)]
pub enum ControlTransfer {}

#[derive(Debug)]
pub enum Invoke {
    Special(u16),
    Virtual(u16),
}

pub fn decode_invoke<I: Iterator<Item = u8>>(opcode: u8, iter: &mut I) -> Invoke {
    use self::Invoke::*;
    let index = read_u16_index(iter);
    match opcode {
        0xb6 => Virtual(index),
        0xb7 => Special(index),
        _ => unimplemented!(),
    }
}

#[derive(Debug)]
pub enum Synchronized {}
