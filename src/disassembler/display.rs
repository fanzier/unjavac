pub use super::compilation_unit::*;
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

pub trait ExtDisplay {
    fn fmt<C>(&self, f: &mut Formatter, unit: &CompilationUnit<C>, indent: usize) -> Result;
}

impl<C: ExtDisplay> Display for CompilationUnit<C> {
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

impl<C: ExtDisplay> Declaration<C> {
    fn fmt<T>(&self, f: &mut Formatter, unit: &CompilationUnit<T>, indent: usize) -> Result {
        match *self {
            Declaration::Field(_) => unimplemented!(),
            Declaration::Method(ref m) => m.fmt(f, unit, indent),
        }
    }
}

impl<C: ExtDisplay> ExtDisplay for Method<C> {
    fn fmt<T>(&self, f: &mut Formatter, unit: &CompilationUnit<T>, indent: usize) -> Result {
        let Method { ref modifiers, ref name, ref signature, ref code } = *self;
        Modifier::fmt_modifiers(f, modifiers)?;
        write!(f, "{}: {}", name, signature)?;
        if let Some(ref code) = *code {
            writeln!(f, " {{")?;
            {
                let indent = indent + 4;
                code.fmt(f, unit, indent)?;
            }
            writeln!(f, "}}")
        } else {
            writeln!(f, ";")
        }
    }
}

impl ExtDisplay for Code {
    fn fmt<T>(&self, f: &mut Formatter, unit: &CompilationUnit<T>, indent: usize) -> Result {
        for &(pc, ref instruction) in &self.instructions {
            write!(f, "{:#6X}: ", pc)?;
            instruction.fmt(f, unit)?;
            newline(f, indent)?;
        }
        Ok(())
    }
}

impl Instruction {
    pub fn fmt<C>(&self, f: &mut Formatter, unit: &CompilationUnit<C>) -> Result {
        match *self {
            Instruction::Nop => write!(f, "nop"),
            Instruction::Load(ref rvalue) => {
                write!(f, "load ")?;
                rvalue.fmt(f, unit)
            }
            Instruction::Store(ref lvalue) => {
                write!(f, "store ")?;
                lvalue.fmt(f, unit)
            }
            Instruction::Invoke(ref invoke) => invoke.fmt(f, unit),
            Instruction::Return(val) => {
                write!(f, "return {}", if val.is_some() { "value" } else { "void" })
            }
            Instruction::Jump(ref jump) => write!(f, "{}", jump),
            Instruction::Arithm(ref arithm) => write!(f, "{}", arithm),
            _ => unimplemented!(),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Instruction::Nop => write!(f, "nop"),
            Instruction::Load(ref rvalue) => write!(f, "load {}", rvalue),
            Instruction::Store(ref lvalue) => write!(f, "store {}", lvalue),
            Instruction::Invoke(ref invoke) => write!(f, "invoke {}", invoke),
            Instruction::Return(val) => {
                write!(f, "return {}", if val.is_some() { "value" } else { "void" })
            }
            Instruction::Jump(ref jump) => write!(f, "{}", jump),
            Instruction::Arithm(ref arithm) => write!(f, "{}", arithm),
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

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Literal::NullReference => write!(f, "null"),
            Literal::Byte(i) => write!(f, "{}: byte", i),
            Literal::Short(i) => write!(f, "{}: short", i),
            Literal::Integer(i) => write!(f, "{}: int", i),
            Literal::Long(i) => write!(f, "{}: long", i),
            // Literal::Float(d) => write!(f, "{}: float", d),
            // Literal::Double(d) => write!(f, "{}: double", d),
            Literal::String(ref s) => write!(f, r#""{}": String"#, s),
        }
    }
}

impl Display for LValue {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            LValue::Local(i) => write!(f, "local_{}", i),
            LValue::Stack(i) => write!(f, "stack[{}]", i),
            LValue::StaticField { field_ref } => write!(f, "static field {}", field_ref),
            LValue::InstanceField { object_stack_index, field_ref } => {
                write!(f,
                       " field {} of stack[{}]",
                       field_ref,
                       object_stack_index + 1)
            }
        }
    }
}

impl LValue {
    pub fn fmt<C>(&self, f: &mut Formatter, unit: &CompilationUnit<C>) -> Result {
        match *self {
            LValue::Local(i) => write!(f, "local_{}", i),
            LValue::Stack(i) => write!(f, "stack[{}]", i),
            LValue::StaticField { field_ref } => {
                let field = &unit.metadata.field_refs[&field_ref];
                let class = &unit.metadata.class_refs[&field.class_ref];
                write!(f, "{}.{}: {}", &class.0, field.name, field.typ)
            }
            LValue::InstanceField { object_stack_index, field_ref } => {
                let field = &unit.metadata.field_refs[&field_ref];
                let class = &unit.metadata.class_refs[&field.class_ref];
                write!(f,
                       "(stack[{}]: {}).{}: {}",
                       object_stack_index,
                       &class.0,
                       field.name,
                       field.typ)
            }
        }
    }
}

impl Display for RValue {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            RValue::Constant(ref constant) => write!(f, "{}", constant),
            RValue::ConstantRef { const_ref } => write!(f, "constant #{}", const_ref),
            RValue::LValue(ref lvalue) => write!(f, "{}", lvalue),
        }
    }
}

impl RValue {
    pub fn fmt<C>(&self, f: &mut Formatter, unit: &CompilationUnit<C>) -> Result {
        match *self {
            RValue::Constant(ref constant) => write!(f, "{}", constant),
            RValue::ConstantRef { const_ref } => {
                let constant = &unit.metadata.literals[&const_ref];
                write!(f, "{}", constant)
            }
            RValue::LValue(ref lvalue) => write!(f, "{}", lvalue),
        }
    }
}

impl Display for Invoke {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self.kind {
            InvokeKind::Virtual => write!(f, "virtual")?,
            InvokeKind::Special => write!(f, "special")?,
            InvokeKind::Static => write!(f, "static")?,
        };
        write!(f, " {}", self.method_index)
    }
}

impl Invoke {
    pub fn fmt<C>(&self, f: &mut Formatter, unit: &CompilationUnit<C>) -> Result {
        match self.kind {
            InvokeKind::Virtual => write!(f, "invoke virtual")?,
            InvokeKind::Special => write!(f, "invoke special")?,
            InvokeKind::Static => write!(f, "invoke static")?,
        };
        let method_ref = &unit.metadata.method_refs[&self.method_index];
        let class = &unit.metadata.class_refs[&method_ref.class_ref].0;
        write!(f,
               " {}.{}: {}",
               class,
               method_ref.name,
               method_ref.signature)
    }
}

impl Display for Jump {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "jump to {:#X}", self.address)?;
        if let Some(condition) = self.condition {
            write!(f, " if {}", condition)?;
        }
        Ok(())
    }
}

impl Display for JumpCondition {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use self::JumpCondition::*;
        match *self {
            CmpZero(ord) => write!(f, "stack[-1] {} 0", ord),
            Cmp(ord) => write!(f, "stack[-2] {} stack[-1]", ord),
            CmpRef(eq) => write!(f, "stack[-2] {} stack[-1]", eq),
        }
    }
}

impl Display for Ordering {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use self::Ordering::*;
        write!(f,
               "{}",
               match *self {
                   EQ => "==",
                   NE => "!=",
                   LT => "<",
                   GE => ">=",
                   GT => ">",
                   LE => "<=",
               })
    }
}

impl Display for Arithm {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Arithm::UnaryOp(unary_op) => write!(f, "{}", unary_op),
            Arithm::BinaryOp(binary_op) => write!(f, "{}", binary_op),
            Arithm::IncreaseLocal { local_index, increase } => {
                write!(f, "increase local_{} by {}", local_index, increase)
            }
        }
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            UnaryOp::Neg => write!(f, "neg"),
        }
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use self::BinaryOp::*;
        write!(f,
               "{}",
               match *self {
                   Add => "add",
                   Sub => "sub",
                   Mul => "mul",
                   Div => "div",
                   Rem => "rem",
                   Shl => "shl",
                   Shr => "shr",
                   Ushr => "ushr",
                   And => "and",
                   Or => "or",
                   Xor => "xor",
               })
    }
}
