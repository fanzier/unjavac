pub use disassembler::types::*;
use std::fmt::*;
use pretty::*;

impl<C> Display for CompilationUnit<C>
    where C: PrettyWith<CompilationUnit<C>>
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        writeln!(f, "{}", self.pretty().render_string(120))
    }
}

impl<C, T> PrettyWith<T> for CompilationUnit<C>
    where C: PrettyWith<CompilationUnit<C>>
{
    fn pretty_with(&self, _: &T) -> Doc {
        let modifiers = self.modifiers.pretty();
        let first = modifiers + format!(" {} {} {{", self.typ, self.name);
        let declarations =
            self.declarations.iter().map(|declaration| declaration.pretty_with(self));
        let declarations = newline() + intersperse(declarations, newline());
        first + declarations.nest(4) + newline() + '}'
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

impl<T> PrettyWith<T> for Vec<Modifier> {
    fn pretty_with(&self, _: &T) -> Doc {
        intersperse(self.iter().map(doc), ' ')
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

impl PrettyWith<str> for Signature {
    fn pretty_with(&self, name: &str) -> Doc {
        let params = self.parameters.iter().map(|&(ref name, ref typ)| if name.is_empty() {
                                                    doc(typ)
                                                } else {
                                                    doc(typ) + ' ' + name
                                                });
        group(doc(&self.return_type) + spaceline() + name + tupled(params))
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.pretty_with("").render_string(None))
    }
}

impl<T, C> PrettyWith<CompilationUnit<T>> for Declaration<C>
    where C: PrettyWith<CompilationUnit<T>>
{
    fn pretty_with(&self, unit: &CompilationUnit<T>) -> Doc {
        match *self {
            Declaration::Field(_) => unimplemented!(),
            Declaration::Method(ref m) => m.pretty_with(unit),
        }
    }
}

impl<C, T> PrettyWith<CompilationUnit<T>> for Method<C>
    where C: PrettyWith<CompilationUnit<T>>
{
    fn pretty_with(&self, unit: &CompilationUnit<T>) -> Doc {
        let mut result = self.modifiers.pretty() + ' ';
        result += self.signature.pretty_with(&self.name);
        if let Some(ref code) = self.code {
            result += " {";
            result += nest(4, newline() + code.pretty_with(unit));
            result += newline() + "}";
        } else {
            result += ";"
        }
        result
    }
}

impl<T> PrettyWith<CompilationUnit<T>> for Code {
    fn pretty_with(&self, unit: &CompilationUnit<T>) -> Doc {
        let docs = self.instructions.iter().map(|&(pc, ref instruction)| {
                                                    doc(format!("{:#6X}: ", pc)) +
                                                    instruction.pretty_with(unit)
                                                });
        intersperse(docs, newline())
    }
}

impl<T> PrettyWith<CompilationUnit<T>> for Instruction {
    fn pretty_with(&self, unit: &CompilationUnit<T>) -> Doc {
        match *self {
            Instruction::Nop => doc("nop"),
            Instruction::Load(ref rvalue) => doc("load ") + rvalue.pretty_with(unit),
            Instruction::Store(ref lvalue) => doc("store ") + lvalue.pretty_with(unit),
            Instruction::Invoke(ref invoke) => invoke.pretty_with(unit),
            Instruction::Return(val) => {
                doc(format!("return {}", if val.is_some() { "value" } else { "void" }))
            }
            Instruction::Jump(ref jump) => doc(format!("{}", jump)),
            Instruction::Arithm(ref arithm) => doc(format!("{}", arithm)),
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
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::Byte(i) => write!(f, "{}", i),
            Literal::Short(i) => write!(f, "{}", i),
            Literal::Integer(i) => write!(f, "{}", i),
            Literal::Long(i) => write!(f, "{}L", i),
            // Literal::Float(d) => write!(f, "{}f", d),
            // Literal::Double(d) => write!(f, "{}d", d),
            Literal::String(ref s) => write!(f, r#""{}""#, s),
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

impl<T> PrettyWith<CompilationUnit<T>> for LValue {
    fn pretty_with(&self, unit: &CompilationUnit<T>) -> Doc {
        match *self {
                LValue::Local(i) => format!("local_{}", i),
                LValue::Stack(i) => format!("stack[{}]", i),
                LValue::StaticField { field_ref } => {
                    let field = &unit.metadata.field_refs[&field_ref];
                    let class = &unit.metadata.class_refs[&field.class_ref];
                    format!("{}.{}: {}", &class.0, field.name, field.typ)
                }
                LValue::InstanceField { object_stack_index, field_ref } => {
                    let field = &unit.metadata.field_refs[&field_ref];
                    let class = &unit.metadata.class_refs[&field.class_ref];
                    format!("(stack[{}]: {}).{}: {}",
                            object_stack_index,
                            &class.0,
                            field.name,
                            field.typ)
                }
            }
            .into()
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

impl<T> PrettyWith<CompilationUnit<T>> for RValue {
    fn pretty_with(&self, unit: &CompilationUnit<T>) -> Doc {
        match *self {
                RValue::Constant(ref constant) => format!("{}", constant),
                RValue::ConstantRef { const_ref } => {
                    let constant = &unit.metadata.literals[&const_ref];
                    format!("{}", constant)
                }
                RValue::LValue(ref lvalue) => format!("{}", lvalue),
            }
            .into()
    }
}

impl Display for Invoke {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let kind = match self.kind {
            InvokeKind::Virtual => "virtual",
            InvokeKind::Special => "special",
            InvokeKind::Static => "static",
        };
        write!(f, "{} {}", kind, self.method_index)
    }
}

impl<T> PrettyWith<CompilationUnit<T>> for Invoke {
    fn pretty_with(&self, unit: &CompilationUnit<T>) -> Doc {
        let kind = match self.kind {
            InvokeKind::Virtual => "invoke virtual",
            InvokeKind::Special => "invoke special",
            InvokeKind::Static => "invoke static",
        };
        let method_ref = &unit.metadata.method_refs[&self.method_index];
        let class = &unit.metadata.class_refs[&method_ref.class_ref].0;
        doc(kind) + ' ' + class + '.' + &method_ref.name + ": " + &method_ref.signature
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

impl<T> PrettyWith<T> for JumpCondition {
    fn pretty_with(&self, _: &T) -> Doc {
        doc(format!("{}", self))
    }
}

impl Ordering {
    pub fn to_str(&self) -> &'static str {
        match *self {
            Ordering::EQ => "==",
            Ordering::NE => "!=",
            Ordering::LT => "<",
            Ordering::GE => ">=",
            Ordering::GT => ">",
            Ordering::LE => "<=",
        }
    }
}

impl Display for Ordering {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.to_str())
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
