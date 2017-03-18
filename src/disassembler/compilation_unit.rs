pub use super::super::classfile::parser::*;
pub use super::instructions::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct CompilationUnit<C> {
    pub typ: UnitType,
    pub modifiers: Vec<Modifier>,
    pub name: String,
    pub declarations: Vec<Declaration<C>>,
    pub java_constants: HashMap<u16, JavaConstant>,
    pub string_constants: HashMap<u16, String>,
    pub class_refs: HashMap<u16, ClassRef>,
    pub field_refs: HashMap<u16, FieldRef>,
    pub method_refs: HashMap<u16, MethodRef>,
    pub name_refs: HashMap<u16, NameRef>,
}

impl<C> CompilationUnit<C> {
    pub fn lookup_string(&self, index: u16) -> &str {
        self.string_constants.get(&index).unwrap()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum UnitType {
    Class,
    Interface,
    Enum,
}

#[derive(Copy, Clone, Debug)]
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
pub enum Declaration<C> {
    Field(Field),
    Method(Method<C>)
}

#[derive(Debug)]
pub struct Field {
    pub modifiers: Vec<Modifier>,
    pub name: String,
    pub typ: Type,
}

#[derive(Debug)]
pub struct Method<C> {
    pub modifiers: Vec<Modifier>,
    pub name: String,
    pub signature: Signature,
    pub code: Option<C>,
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
pub enum Descriptor {
    Signature(Signature),
    Type(Type),
}

#[derive(Debug)]
pub enum JavaConstant {
    NullReference,
    Byte(i8),
    Short(i16),
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
}

#[derive(Debug)]
pub struct ClassRef(pub String);

#[derive(Debug)]
pub struct FieldRef {
    pub class_ref: u16,
    pub name: String,
    pub typ: Type,
}

#[derive(Debug)]
pub struct MethodRef {
    pub class_ref: u16,
    pub name: String,
    pub signature: Signature,
}

#[derive(Debug)]
pub struct NameRef {
    pub name: String,
    pub typ: Descriptor,
}

#[derive(Debug)]
pub struct Code {
    // TODO: Exception handlers
    pub instructions: Vec<(u16, Instruction)>,
}