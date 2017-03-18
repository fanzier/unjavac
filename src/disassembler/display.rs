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

impl Display for CompilationUnit<Code> {
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
            write!(f, " {{")?;
            {
                let indent = indent + 4;
                code.fmt(f, unit, indent)?;
            }
            newline(f, indent)?;
            writeln!(f, "}}")
        } else {
            writeln!(f, ";")
        }
    }
}

impl ExtDisplay for Code {
    fn fmt<T>(&self, f: &mut Formatter, unit: &CompilationUnit<T>, indent: usize) -> Result {
        for &(pc, ref instruction) in &self.instructions {
            newline(f, indent)?;
            write!(f, "{:#6X}: ", pc)?;
            instruction.fmt(f, unit)?;
        }
        Ok(())
    }
}

impl Instruction {
    pub fn fmt<C>(&self, f: &mut Formatter, unit: &CompilationUnit<C>) -> Result {
        match *self {
            Instruction::Nop => write!(f, "nop"),
            Instruction::Cpy(ref cpy) => cpy.fmt(f, unit),
            Instruction::Invoke(ref invoke) => invoke.fmt(f, unit),
            Instruction::Return => write!(f, "return"),
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
            Instruction::Cpy(ref cpy) => write!(f, "copy {}", cpy),
            Instruction::Invoke(ref invoke) => write!(f, "invoke {}", invoke),
            Instruction::Return => write!(f, "return"),
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

impl Display for JavaConstant {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            JavaConstant::NullReference => write!(f, "null"),
            JavaConstant::Byte(i) => write!(f, "{}: byte", i),
            JavaConstant::Short(i) => write!(f, "{}: short", i),
            JavaConstant::Integer(i) => write!(f, "{}: int", i),
            JavaConstant::Long(i) => write!(f, "{}: long", i),
            JavaConstant::Float(d) => write!(f, "{}: float", d),
            JavaConstant::Double(d) => write!(f, "{}: double", d),
            JavaConstant::String(ref s) => write!(f, r#""{}": String"#, s),
        }
    }
}

impl Display for Cpy {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{} -> {}", self.from, self.to)
    }
}

impl Cpy {
    pub fn fmt<C>(&self, f: &mut Formatter, unit: &CompilationUnit<C>) -> Result {
        write!(f, "copy ")?;
        self.from.fmt(f, unit)?;
        write!(f, " -> ")?;
        self.to.fmt(f, unit)
    }
}

impl Display for LValue {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            LValue::PushStack => write!(f, "push onto stack"),
            LValue::Local(i) => write!(f, "local_{}", i),
            LValue::Stack(i) => write!(f, "stack[-{}]", i + 1),
            LValue::StaticField { field_ref } => write!(f, "static field {}", field_ref),
            LValue::InstanceField { object_stack_index, field_ref } => {
                write!(f,
                       " field {} of stack[-{}]",
                       field_ref,
                       object_stack_index + 1)
            }
        }
    }
}

impl LValue {
    pub fn fmt<C>(&self, f: &mut Formatter, unit: &CompilationUnit<C>) -> Result {
        match *self {
            LValue::PushStack => write!(f, "push onto stack"),
            LValue::Local(i) => write!(f, "local_{}", i),
            LValue::Stack(i) => write!(f, "stack[-{}]", i + 1),
            LValue::StaticField { field_ref } => {
                let field = unit.field_refs.get(&field_ref).unwrap();
                let class = &unit.class_refs.get(&field.class_ref).unwrap();
                write!(f, "{}.{}: {}", &class.0, field.name, field.typ)
            }
            LValue::InstanceField { object_stack_index, field_ref } => {
                let field = unit.field_refs.get(&field_ref).unwrap();
                let class = &unit.class_refs.get(&field.class_ref).unwrap();
                write!(f,
                       "(stack[-{}]: {}).{}: {}",
                       object_stack_index + 1,
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
            RValue::Local(i) => write!(f, "local_{}", i),
            RValue::Stack(i) => write!(f, "stack[-{}]", i + 1),
            RValue::StaticField { field_ref } => write!(f, "static field {}", field_ref),
            RValue::InstanceField { object_stack_index, field_ref } => {
                write!(f,
                       " field {} of stack[-{}]",
                       field_ref,
                       object_stack_index + 1)
            }
        }
    }
}

impl RValue {
    pub fn fmt<C>(&self, f: &mut Formatter, unit: &CompilationUnit<C>) -> Result {
        match *self {
            RValue::Constant(ref constant) => write!(f, "{}", constant),
            RValue::ConstantRef { const_ref } => {
                let constant = unit.java_constants.get(&const_ref).unwrap();
                write!(f, "{}", constant)
            }
            RValue::Local(i) => write!(f, "local_{}", i),
            RValue::Stack(i) => write!(f, "stack[-{}]", i + 1),
            RValue::StaticField { field_ref } => {
                let field = unit.field_refs.get(&field_ref).unwrap();
                let class = &unit.class_refs.get(&field.class_ref).unwrap();
                write!(f, "{}.{}: {}", &class.0, field.name, field.typ)
            }
            RValue::InstanceField { object_stack_index, field_ref } => {
                let field = unit.field_refs.get(&field_ref).unwrap();
                let class = &unit.class_refs.get(&field.class_ref).unwrap();
                write!(f,
                       "(stack[-{}]: {}).{}: {}",
                       object_stack_index + 1,
                       &class.0,
                       field.name,
                       field.typ)
            }
        }
    }
}

impl Display for Invoke {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let index = match *self {
            Invoke::Virtual(index) => {
                write!(f, "invoke virtual")?;
                index
            }
            Invoke::Special(index) => {
                write!(f, "invoke special")?;
                index
            }
            Invoke::Static(index) => {
                write!(f, "invoke static")?;
                index
            }
        };
        write!(f, " {}", index)
    }
}

impl Invoke {
    pub fn fmt<C>(&self, f: &mut Formatter, unit: &CompilationUnit<C>) -> Result {
        let index = match *self {
            Invoke::Virtual(index) => {
                write!(f, "invoke virtual")?;
                index
            }
            Invoke::Special(index) => {
                write!(f, "invoke special")?;
                index
            }
            Invoke::Static(index) => {
                write!(f, "invoke special")?;
                index
            }
        };
        let method_ref = unit.method_refs.get(&index).unwrap();
        let class = &unit.class_refs.get(&method_ref.class_ref).unwrap().0;
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

