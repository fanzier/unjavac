use decompiler::cfg::*;
use decompiler::types::*;
use disassembler::instructions::*;

pub fn handle_constructors(unit: &mut CompilationUnit<Cfg<Statement, Expr>>) {
    let class_name = unit.name.to_owned();
    unit.declarations.iter_mut().for_each(|declaration| {
        let replacement = match *declaration {
            Declaration::Method(ref mut m) => if m.name == "<init>" {
                let modifiers = m.modifiers.clone();
                let parameters = m.signature.parameters.clone();
                let mut code = m.code.clone().expect("abstract contstructor");
                let mut visitor = ConstructorVisitor {
                    class_name: &class_name,
                };
                code.map(|stmts| {
                    stmts
                        .iter_mut()
                        .for_each(|stmt| visitor.visit_statement(stmt))
                });
                Some(Declaration::Constructor(Constructor {
                    modifiers,
                    parameters,
                    code,
                }))
            } else {
                None
            },
            _ => None,
        };
        // FIXME: Fix this once non-lexical lifetimes are available
        if let Some(dec) = replacement {
            *declaration = dec;
        }
    });
}

pub struct ConstructorVisitor<'a> {
    class_name: &'a str,
}

impl<'a> Visitor for ConstructorVisitor<'a> {
    fn visit_statement(&mut self, stmt: &mut Statement) {
        let replacement = if let Statement::Expr(Expr::Invoke(
            Some(ref expr),
            MethodRef {
                name: ref method_name,
                ..
            },
            ClassRef(ref class_name),
            ref args,
        )) = *stmt
        {
            assert!(
                match **expr {
                    Expr::This => true,
                    _ => false,
                },
                "constructor call not on `this`!"
            );
            if method_name == "<init>" {
                if self.class_name == class_name {
                    Some(Statement::ThisCall(args.clone()))
                } else {
                    Some(Statement::SuperCall(args.clone()))
                }
            } else {
                None
            }
        } else {
            None
        };
        if let Some(ctor_call) = replacement {
            *stmt = ctor_call;
        }
    }
}
