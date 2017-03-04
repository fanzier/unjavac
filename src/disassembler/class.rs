pub use super::super::classfile::parser::*;

#[derive(Debug)]
pub struct CompilationUnit {
    pub typ: UnitType,
    pub modifiers: Vec<Modifier>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug)]
pub enum UnitType {
    Class,
    Interface,
    Enum,
}

#[derive(Debug)]
pub enum Modifier {
    Public,
    Protected,
    Private,
    Static,
    Abstract,
    Final,
    Native,
    Synchronized,
    Transient,
    Volatile,
    Strictfp,
}

#[derive(Debug)]
pub enum Declaration {
    Field {
        modifiers: Vec<Modifier>,
        name: String,
        typ: Type,
    },
    Method {
        modifiers: Vec<Modifier>,
        name: String,
        signature: Signature,
        code: Option<Code>,
    },
}

#[derive(Debug)]
pub enum Type {
    Void,
    Boolean,
    Byte,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,
    Array(Box<Type>),
    Reference(String),
}

#[derive(Debug)]
pub struct Signature {
    pub parameters: Vec<Type>,
    pub return_type: Type,
}

#[derive(Debug)]
pub struct Code {
    // TODO
}
