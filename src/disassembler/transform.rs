pub use super::super::classfile::parser::*;
pub use super::class::*;
pub use super::disassembler::*;

pub fn transform(class_file: ClassFile) -> CompilationUnit {
    let declarations =
        class_file.methods.iter().map(|method| transform_method(&class_file, method)).collect();
    // TODO: Also process fields, inner classes etc.
    CompilationUnit {
        typ: if class_file.access_flags.contains(ACC_INTERFACE) {
            UnitType::Interface
        } else if class_file.access_flags.contains(ACC_ENUM) {
            UnitType::Enum
        } else {
            UnitType::Class
        },
        modifiers: class_flags_to_modifiers(&class_file.access_flags),
        declarations: declarations,
    }
}

fn class_flags_to_modifiers(flags: &AccessFlags) -> Vec<Modifier> {
    let mut modifiers = vec![];
    if flags.contains(ACC_PUBLIC) {
        modifiers.push(Modifier::Public);
    }
    if flags.contains(ACC_PROTECTED) {
        modifiers.push(Modifier::Protected);
    }
    if flags.contains(ACC_PRIVATE) {
        modifiers.push(Modifier::Private);
    }
    if flags.contains(ACC_STATIC) {
        modifiers.push(Modifier::Static);
    }
    if flags.contains(ACC_ABSTRACT) {
        modifiers.push(Modifier::Abstract);
    }
    if flags.contains(ACC_FINAL) {
        modifiers.push(Modifier::Final);
    }
    modifiers
}

fn transform_method(class_file: &ClassFile, method: &MethodInfo) -> Declaration {
    let ref constant_pool = class_file.constant_pool;
    let mut code = None;
    for attribute in &method.attributes {
        if constant_pool.lookup_string(attribute.name_index) == "Code" {
            let code_attribute = parse_code_attribute(&attribute.info).unwrap();
            println!("Parsed code attribute: {:#?}", code_attribute);
            let disassembly = disassemble(class_file, code_attribute);
            code = Some(disassembly);
            break;
        }
    }
    Declaration::Method {
        modifiers: method_flags_to_modifiers(&method.access_flags),
        name: constant_pool.lookup_string(method.name_index).to_owned(),
        signature: descriptor_to_signature(constant_pool.lookup_string(method.descriptor_index)),
        code: code,
    }
}

fn method_flags_to_modifiers(flags: &AccessFlags) -> Vec<Modifier> {
    let mut modifiers = vec![];
    if flags.contains(ACC_PUBLIC) {
        modifiers.push(Modifier::Public);
    }
    if flags.contains(ACC_PROTECTED) {
        modifiers.push(Modifier::Protected);
    }
    if flags.contains(ACC_PRIVATE) {
        modifiers.push(Modifier::Private);
    }
    if flags.contains(ACC_STATIC) {
        modifiers.push(Modifier::Static);
    }
    if flags.contains(ACC_ABSTRACT) {
        modifiers.push(Modifier::Abstract);
    }
    if flags.contains(ACC_FINAL) {
        modifiers.push(Modifier::Final);
    }
    // Method specific flags:
    if flags.contains(ACC_SYNCHRONIZED) {
        modifiers.push(Modifier::Synchronized);
    }
    if flags.contains(ACC_NATIVE) {
        modifiers.push(Modifier::Native);
    }
    if flags.contains(ACC_STRICT) {
        modifiers.push(Modifier::Strictfp);
    }
    modifiers
}

fn descriptor_to_signature(descriptor: &str) -> Signature {
    let mut chars = descriptor.chars().peekable();
    let mut params = vec![];
    let next = chars.next().unwrap();
    if next != '(' {
        panic!("Expected open paren at beginning of method descriptor: {:?}",
               descriptor);
    }
    loop {
        let lookahead = *chars.peek().unwrap();
        if lookahead != ')' {
            params.push(descriptor_to_type(&mut chars));
        }
        let next = chars.next().unwrap();
        match next {
            ')' => break,
            ',' => (),
            _ => panic!("Expected ')' or ',' in method descriptor: {:?}", descriptor),
        }
    }
    let return_type = descriptor_to_type(&mut chars);
    Signature {
        parameters: params,
        return_type: return_type,
    }
}

fn descriptor_to_type<I: Iterator<Item = char>>(chars: &mut I) -> Type {
    let next = chars.next().unwrap();
    match next {
        'B' => Type::Byte,
        'C' => Type::Char,
        'D' => Type::Double,
        'F' => Type::Float,
        'I' => Type::Int,
        'J' => Type::Long,
        'L' => {
            let mut class_name = String::new();
            for ch in chars {
                if ch == ';' {
                    break;
                }
                class_name.push(ch);
            }
            Type::Reference(class_name)
        }
        'S' => Type::Short,
        'V' => Type::Void,
        'Z' => Type::Boolean,
        '[' => Type::Array(Box::new(descriptor_to_type(chars))),
        _ => panic!("Invalid start of type descriptor: {:?}", next),
    }
}
