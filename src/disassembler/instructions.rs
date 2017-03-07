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
    Cpy(Cpy),
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
        0x01...0x35 | 0xb2 | 0xb4 => Cpy(decode_load(opcode, iter)),
        0x36...0x56 | 0xb3 | 0xb5 => Cpy(decode_store(opcode, iter)),
        0x57...0x5f => unimplemented!(), // stack Stack
        0x60...0x84 => unimplemented!(), // arithmetic
        0x85...0x93 => unimplemented!(), // type conversion
        0x94...0x98 => unimplemented!(), // comparison (arithmetic)
        0x99...0xab => unimplemented!(), // control flow
        0xac...0xb1 => Return,
        0xb2...0xb5 => Cpy(decode_store(opcode, iter)), // get/put field
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
pub struct Cpy {
    pub to: LValue,
    pub from: RValue,
}

#[derive(Debug)]
pub enum LValue {
    PushStack,
    Local(usize),
    Stack(usize), // Stack(i): i-th element from top of stack, i.e. Stack(0) is top of stack
    StaticField { field_ref: u16 },
    InstanceField {
        object_stack_index: usize,
        field_ref: u16,
    },
}

#[derive(Debug)]
pub enum RValue {
    Constant { const_ref: u16 },
    Local(usize),
    Stack(usize), // Stack(i): i-th element from top of stack, i.e. Stack(0) is top of stack
    StaticField { field_ref: u16 },
    InstanceField {
        object_stack_index: usize,
        field_ref: u16,
    },
}

pub fn decode_load<I: Iterator<Item = u8>>(opcode: u8, iter: &mut I) -> Cpy {
    let origin = match opcode {
        0x12 => {
            let index = iter.next().unwrap();
            RValue::Constant { const_ref: index as u16 }
        }
        0x1a...0x1d => RValue::Local((opcode - 0x1a) as usize),
        0x2a...0x2d => RValue::Local((opcode - 0x2a) as usize),
        0xb2 => {
            //getstatic
            let index = read_u16_index(iter);
            RValue::StaticField { field_ref: index }
        }
        0xb4 => {
            //getfield
            let index = read_u16_index(iter);
            RValue::InstanceField {
                object_stack_index: 0,
                field_ref: index,
            }
        }
        _ => unimplemented!(),
    };
    Cpy {
        to: LValue::PushStack,
        from: origin,
    }
}

pub fn decode_store<I: Iterator<Item = u8>>(opcode: u8, iter: &mut I) -> Cpy {
    let target = match opcode {
        0x3b...0x3e => LValue::Local((opcode - 0x3b) as usize),
        0xb3 => {
            // putstatic
            let index = read_u16_index(iter);
            LValue::StaticField { field_ref: index }
        }
        0xb5 => {
            // putfield
            let index = read_u16_index(iter);
            LValue::InstanceField {
                object_stack_index: 1,
                field_ref: index,
            }
        }
        _ => unimplemented!(),
    };
    Cpy {
        to: target,
        from: RValue::Stack(0),
    }
}

#[derive(Debug)]
pub enum Arithm {}

#[derive(Debug)]
pub enum TypeConv {}

#[derive(Debug)]
pub enum ObjManip {}

#[derive(Debug)]
pub enum StackManage {}

#[derive(Debug)]
pub enum ControlTransfer {}

#[derive(Debug)]
pub enum Invoke {
    Special(u16),
    Virtual(u16),
    Static(u16),
}

pub fn decode_invoke<I: Iterator<Item = u8>>(opcode: u8, iter: &mut I) -> Invoke {
    use self::Invoke::*;
    let index = read_u16_index(iter);
    match opcode {
        0xb6 => Virtual(index),
        0xb7 => Special(index),
        0xb8 => Static(index),
        _ => unimplemented!(),
    }
}

#[derive(Debug)]
pub enum Synchronized {}
