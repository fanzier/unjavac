pub use super::super::classfile::parser::*;
pub use super::instructions::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct CompilationUnit<C> {
    pub typ: UnitType,
    pub modifiers: Vec<Modifier>,
    pub name: String,
    pub metadata: Metadata,
    pub declarations: Vec<Declaration<C>>,
}

impl<C> CompilationUnit<C> {
    pub fn lookup_string(&self, index: u16) -> &str {
        &self.metadata.string_constants[&index]
    }

    pub fn map<F, D>(mut self, mut f: F) -> CompilationUnit<D>
        where F: FnMut(C, &Metadata) -> D
    {
        let declarations = {
            let declarations = &mut self.declarations;
            let metadata = &self.metadata;
            declarations.drain(..).map(|d| d.map(|c| f(c, metadata))).collect::<Vec<_>>()
        };
        CompilationUnit {
            typ: self.typ,
            modifiers: self.modifiers,
            name: self.name,
            declarations: declarations,
            metadata: self.metadata,
        }
    }
}

#[derive(Debug, Default)]
pub struct Metadata {
    pub java_constants: HashMap<u16, JavaConstant>,
    pub string_constants: HashMap<u16, String>,
    pub class_refs: HashMap<u16, ClassRef>,
    pub field_refs: HashMap<u16, FieldRef>,
    pub method_refs: HashMap<u16, MethodRef>,
    pub name_refs: HashMap<u16, NameRef>,
}

impl Metadata {
    pub fn new() -> Metadata {
        Metadata::default()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum UnitType {
    Class,
    Interface,
    Enum,
}

#[derive(Copy, Clone, Debug, Hash)]
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
    Method(Method<C>),
}

impl<C> Declaration<C> {
    pub fn map<F, D>(self, f: F) -> Declaration<D>
        where F: FnMut(C) -> D
    {
        match self {
            Declaration::Field(f) => Declaration::Field(f),
            Declaration::Method(Method { modifiers, name, signature, code }) => {
                Declaration::Method(Method {
                                        modifiers: modifiers,
                                        name: name,
                                        signature: signature,
                                        code: code.map(f),
                                    })
            }
        }
    }
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

#[derive(Clone, Debug, Hash, PartialEq)]
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

#[derive(Clone, Debug, Hash)]
pub struct Signature {
    pub parameters: Vec<Type>,
    pub return_type: Type,
}

#[derive(Clone, Debug, Hash)]
pub enum Descriptor {
    Signature(Signature),
    Type(Type),
}

#[derive(Clone, Debug, Hash)]
pub enum JavaConstant {
    NullReference,
    Byte(i8),
    Short(i16),
    Integer(i32),
    Long(i64),
    // TODO: Add these back (requires custom Hash impl):
    // Float(f32),
    // Double(f64),
    String(String),
}

#[derive(Clone, Debug, Hash)]
pub struct ClassRef(pub String);

#[derive(Clone, Debug, Hash)]
pub struct FieldRef {
    pub class_ref: u16,
    pub name: String,
    pub typ: Type,
}

#[derive(Clone, Debug, Hash)]
pub struct MethodRef {
    pub class_ref: u16,
    pub name: String,
    pub signature: Signature,
}

#[derive(Clone, Debug, Hash)]
pub struct NameRef {
    pub name: String,
    pub typ: Descriptor,
}

#[derive(Debug)]
pub struct Code {
    // TODO: Exception handlers
    pub instructions: Vec<(u16, Instruction)>,
}
