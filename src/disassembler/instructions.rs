pub use super::compilation_unit::*;

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
    Jump(Jump),
    Invoke(Invoke),
    Throw,
    Return,
    Synchronized(Synchronized),
}

pub fn decode_instruction<I>(opcode: u8, pc: u16, iter: &mut I) -> Instruction
    where I: Iterator<Item = u8>
{
    use self::Instruction::*;
    println!("Decoding opcode 0x{:x}.", opcode);
    match opcode {
        0x00 => Nop,
        0x01...0x35 | 0xb2 | 0xb4 => Cpy(decode_load(opcode, iter)),
        0x36...0x56 | 0xb3 | 0xb5 => Cpy(decode_store(opcode, iter)),
        0x57...0x5f => unimplemented!(), // stack management
        0x60...0x84 => Arithm(decode_arithm(opcode, iter)), // arithmetic
        0x85...0x93 => unimplemented!(), // type conversion
        0x94...0x98 => unimplemented!(), // comparison (arithmetic)
        0x99...0xab => Jump(decode_jump(opcode, pc, iter)), // control flow
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
    Constant(JavaConstant),
    ConstantRef { const_ref: u16 },
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
        0x02...0x08 => RValue::Constant(JavaConstant::Integer(opcode as i32 - 0x03)),
        0x12 => {
            let index = iter.next().unwrap();
            RValue::ConstantRef { const_ref: index as u16 }
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
pub enum Arithm {
    UnaryOp(UnaryOp),
    BinaryOp(BinaryOp),
    IncreaseLocal { local_index: u8, increase: u8 },
}

#[derive(Copy, Clone, Debug)]
pub enum UnaryOp {
    Neg,
}

#[derive(Copy, Clone, Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Shl,
    Shr,
    Ushr,
    And,
    Or,
    Xor,
}

pub fn decode_arithm<I: Iterator<Item = u8>>(opcode: u8, iter: &mut I) -> Arithm {
    use self::Arithm::*;
    use self::BinaryOp::*;
    use self::UnaryOp::*;
    match opcode {
        0x60...0x63 => BinaryOp(Add),
        0x64...0x67 => BinaryOp(Sub),
        0x68...0x6b => BinaryOp(Mul),
        0x6c...0x6e => BinaryOp(Div),
        0x70...0x73 => BinaryOp(Rem),
        0x74...0x77 => UnaryOp(Neg),
        0x78...0x79 => BinaryOp(Shl),
        0x7a...0x7b => BinaryOp(Shr),
        0x7c...0x7d => BinaryOp(Ushr),
        0x7e...0x7f => BinaryOp(And),
        0x80...0x81 => BinaryOp(Or),
        0x82...0x83 => BinaryOp(Xor),
        0x84 => {
            let index = iter.next().unwrap();
            let increase = iter.next().unwrap();
            IncreaseLocal {
                local_index: index,
                increase: increase,
            }
        }
        _ => unreachable!(),
    }
}

#[derive(Debug)]
pub enum TypeConv {}

#[derive(Debug)]
pub enum ObjManip {}

#[derive(Debug)]
pub enum StackManage {}

#[derive(Debug)]
pub struct Jump {
    pub address: u16,
    pub condition: Option<JumpCondition>,
}

#[derive(Copy, Clone, Debug)]
pub enum JumpCondition {
    CmpZero(Ordering),
    Cmp(Ordering),
    CmpRef(Ordering),
}

#[derive(Copy, Clone, Debug)]
pub enum Ordering {
    EQ,
    NE,
    LT,
    GE,
    GT,
    LE,
}

impl Ordering {
    pub fn from_u8(i: u8) -> Ordering {
        use self::Ordering::*;
        match i {
            0 => EQ,
            1 => NE,
            2 => LT,
            3 => GE,
            4 => GT,
            5 => LE,
            _ => unreachable!(),
        }
    }
}

pub fn decode_jump<I: Iterator<Item = u8>>(opcode: u8, pc: u16, iter: &mut I) -> Jump {
    let offset = read_u16_index(iter) as i16;
    let address = (pc as i32 + offset as i32) as u16;
    let condition = match opcode {
        0x99...0x9e => Some(JumpCondition::CmpZero(Ordering::from_u8(opcode - 0x99))),
        0x9f...0xa4 => Some(JumpCondition::Cmp(Ordering::from_u8(opcode - 0x9f))),
        0xa5...0xa6 => Some(JumpCondition::CmpRef(Ordering::from_u8(opcode - 0x9f))),
        0xa7 => None,
        _ => unimplemented!(),
    };
    Jump {
        address: address,
        condition: condition,
    }
}

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
