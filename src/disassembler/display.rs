pub use super::class::*;
use std::fmt::*;

fn do_indent(f: &mut Formatter, indent: usize) -> Result {
    for _ in 0..indent {
        write!(f, " ")?;
    }
    Ok(())
}

fn newline(f: &mut Formatter, indent: usize) -> Result {
    writeln!(f, "")?;
    do_indent(f, indent)
}

impl Display for CompilationUnit {
    fn fmt(&self, f: &mut Formatter) -> Result {
        Modifier::fmt_modifiers(f, &self.modifiers)?;
        write!(f, "{} {} {{", self.typ, self.name)?;
        for declaration in &self.declarations {
            newline(f, 4)?;
            declaration.fmt(f, self, 4)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

impl Display for Modifier {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let string = match *self {
            Modifier::Public => "public",
            Modifier::Protected => "protected",
            Modifier::Private => "private",
            Modifier::Static => "static",
            Modifier::Abstract => "abstract",
            Modifier::Final => "final",
            Modifier::Native => "native",
            Modifier::Synchronized => "synchronized",
            Modifier::Transient => "transient",
            Modifier::Volatile => "volatile",
            Modifier::Strictfp => "strictfp",
        };
        write!(f, "{}", string)
    }
}

impl Modifier {
    fn fmt_modifiers(f: &mut Formatter, modifiers: &[Modifier]) -> Result {
        for modifier in modifiers {
            write!(f, "{} ", modifier)?;
        }
        Ok(())
    }
}

impl Display for UnitType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let string = match *self {
            UnitType::Class => "class",
            UnitType::Interface => "interface",
            UnitType::Enum => "enum",
        };
        write!(f, "{}", string)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Type::Void => write!(f, "void"),
            Type::Boolean => write!(f, "boolean"),
            Type::Byte => write!(f, "byte"),
            Type::Short => write!(f, "short"),
            Type::Char => write!(f, "char"),
            Type::Int => write!(f, "int"),
            Type::Long => write!(f, "long"),
            Type::Float => write!(f, "float"),
            Type::Double => write!(f, "double"),
            Type::Array(ref ty) => write!(f, "{}[]", ty),
            Type::Reference(ref class) => write!(f, "{}", class),
        }
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "(")?;
        let mut first = true;
        for param in &self.parameters {
            if first {
                write!(f, "{}", param)?;
            } else {
                write!(f, ", {}", param)?;
            }
            first = false;
        }
        write!(f, ") -> {}", self.return_type)
    }
}

impl Declaration {
    pub fn fmt(&self, f: &mut Formatter, unit: &CompilationUnit, indent: usize) -> Result {
        match *self {
            Declaration::Field { .. } => unimplemented!(),
            Declaration::Method { ref modifiers, ref name, ref signature, ref code } => {
                Modifier::fmt_modifiers(f, modifiers)?;
                write!(f, "{}: {}", name, signature)?;
                if let Some(ref code) = *code {
                    write!(f, " {{")?;
                    {
                        let indent = indent + 4;
                        for &(pc, ref instruction) in &code.instructions {
                            newline(f, indent)?;
                            write!(f, "{:#6X}: ", pc)?;
                            instruction.fmt(f, unit)?;
                        }
                    }
                    newline(f, indent)?;
                    writeln!(f, "}}")
                } else {
                    writeln!(f, ";")
                }
            }
        }
    }
}

impl Instruction {
    pub fn fmt(&self, f: &mut Formatter, unit: &CompilationUnit) -> Result {
        match *self {
            Instruction::Nop => write!(f, "nop"),
            Instruction::Load(ref load) => load.fmt(f, unit),
            Instruction::Store(ref store) => write!(f, "{}", store),
            Instruction::ObjManip(ref obj_manip) => obj_manip.fmt(f, unit),
            Instruction::Invoke(ref invoke) => invoke.fmt(f, unit),
            Instruction::Return => write!(f, "return"),
            _ => unimplemented!(),
        }
    }
}

impl Display for Kind {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use self::Kind::*;
        write!(f,
               "{}",
               match *self {
                   B => "byte",
                   C => "char",
                   S => "short",
                   I => "int",
                   L => "long",
                   F => "float",
                   D => "double",
                   A => "reference",
               })
    }
}

impl Display for JavaConstant {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            JavaConstant::Integer(i) => write!(f, "{}: int", i),
            JavaConstant::String(ref s) => write!(f, r#""{}": String"#, s),
        }
    }
}

impl Load {
    pub fn fmt(&self, f: &mut Formatter, unit: &CompilationUnit) -> Result {
        match *self {
            Load::Var(k, i) => write!(f, "load local_{}: {}", i, k),
            Load::Ldc(c) => {
                write!(f,
                       "load constant {}",
                       unit.java_constants.get(&(c as u16)).unwrap())
            }
        }
    }
}

impl Display for Store {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Store::Var(k, i) => write!(f, "store in local_{}: {}", i, k),
        }
    }
}

impl Display for GetOrPut {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            GetOrPut::Get => write!(f, "get"),
            GetOrPut::Put => write!(f, "put"),
        }
    }
}

impl Display for StaticOrField {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            StaticOrField::Static => write!(f, "static"),
            StaticOrField::Field => write!(f, "field"),
        }
    }
}

impl ObjManip {
    pub fn fmt(&self, f: &mut Formatter, unit: &CompilationUnit) -> Result {
        match *self {
            ObjManip::Access(rp, sf, index) => {
                let field_ref = unit.field_refs.get(&index).unwrap();
                let class = &unit.class_refs.get(&field_ref.class_ref).unwrap().0;
                write!(f,
                       "{} {} {}.{}: {}",
                       rp,
                       sf,
                       class,
                       field_ref.name,
                       field_ref.typ)
            }
        }
    }
}

impl Invoke {
    pub fn fmt(&self, f: &mut Formatter, unit: &CompilationUnit) -> Result {
        match *self {
            Invoke::Virtual(index) => {
                let method_ref = unit.method_refs.get(&index).unwrap();
                let class = &unit.class_refs.get(&method_ref.class_ref).unwrap().0;
                write!(f,
                       "invoke virtual {}.{}: {}",
                       class,
                       method_ref.name,
                       method_ref.signature)
            }
            Invoke::Special(index) => {
                let method_ref = unit.method_refs.get(&index).unwrap();
                let class = &unit.class_refs.get(&method_ref.class_ref).unwrap().0;
                write!(f,
                       "invoke special {}.{}: {}",
                       class,
                       method_ref.name,
                       method_ref.signature)
            }
        }
    }
}
