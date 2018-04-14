use decompiler::types::*;
use pretty::*;
use std::cmp::Ordering;

trait HasPrecedence {
    fn precedence(&self) -> Precedence;
}

#[derive(Copy, Clone, Debug, Hash, PartialEq)]
enum Precedence {
    Least = 0,
    Assign = 10,  // =, +=, -=, ...
    Ternary = 20, // ? :
    LogOr = 30,   // ||
    LogAnd = 31,  // &&
    Cmp = 40,     // ==, !=, <=, >=, >, <, instanceof
    BitOp = 50,   // &, |, ^, >>, >>>, <<
    Add = 60,     // +, -
    Mul = 70,     // *, /, %
    Unary = 80,   // -, +, !, ~, ++, --
    Access = 90,  // (), [], .
    Tightest = 100,
}

impl PartialOrd<Precedence> for Precedence {
    fn partial_cmp(&self, other: &Precedence) -> Option<Ordering> {
        if *self == *other {
            return Some(Ordering::Equal);
        }
        if *self == Precedence::Tightest || *other == Precedence::Least {
            return Some(Ordering::Greater);
        }
        if *self == Precedence::Least || *other == Precedence::Tightest {
            return Some(Ordering::Less);
        }
        // bit operations have confusing precedence, hence always parenthesize:
        if *self == Precedence::BitOp || *other == Precedence::BitOp {
            return None;
        }
        // otherwise compare numeric precedence value
        (*self as usize).partial_cmp(&(*other as usize))
    }
}

fn parens_if<T>(e: &T, outer: Precedence, parens_if_equal_prec: bool) -> Doc
where
    T: HasPrecedence + Pretty,
{
    let doc = e.pretty();
    let inner = e.precedence();
    if inner > outer {
        return doc;
    }
    if inner == outer && !parens_if_equal_prec {
        return doc;
    }
    parens(doc)
}

fn parens(d: Doc) -> Doc {
    doc("(") + d + ")"
}

impl HasPrecedence for Assignable {
    fn precedence(&self) -> Precedence {
        match *self {
            Assignable::Variable(..) => Precedence::Tightest,
            Assignable::Field { .. } | Assignable::ArrayAccess { .. } => Precedence::Access,
        }
    }
}

impl HasPrecedence for Expr {
    fn precedence(&self) -> Precedence {
        match *self {
            Expr::Assignable(ref v) => v.precedence(),
            Expr::UnaryOp(..) => Precedence::Unary,
            Expr::BinaryOp(op, _, _) => match op {
                BinOp::Cmp(_) => Precedence::Cmp,
                BinOp::Add | BinOp::Sub => Precedence::Add,
                BinOp::Mul | BinOp::Div | BinOp::Rem => Precedence::Mul,
                BinOp::LogAnd => Precedence::LogAnd,
                BinOp::LogOr => Precedence::LogOr,
                BinOp::Shl
                | BinOp::Shr
                | BinOp::Ushr
                | BinOp::BitAnd
                | BinOp::BitOr
                | BinOp::BitXor => Precedence::BitOp,
            },
            Expr::IfThenElse { .. } => Precedence::Ternary,
            Expr::Invoke(..) | Expr::Assign { .. } => Precedence::Assign,
            Expr::Literal(_) | Expr::New { .. } | Expr::This | Expr::Super => Precedence::Tightest,
        }
    }
}

impl<T> PrettyWith<T> for Expr {
    fn pretty_with(&self, _: &T) -> Doc {
        match *self {
            Expr::Literal(ref literal) => format!("{}", literal).into(),
            Expr::Assignable(ref v) => v.pretty(),
            Expr::UnaryOp(op, ref e) => Doc::from(op) + parens_if(&**e, self.precedence(), true),
            Expr::BinaryOp(op, ref e1, ref e2) => {
                (parens_if(&**e1, self.precedence(), true) + spaceline()
                    + group(format!("{} ", op).into())
                    + parens_if(&**e2, self.precedence(), true))
                    .group()
            }
            Expr::IfThenElse { .. } => unimplemented!(),
            Expr::Invoke(ref this, ref method, ref class, ref args) => {
                let result = if let Some(ref this) = *this {
                    this.pretty()
                } else {
                    class.0.to_owned().into()
                };
                let result = result + format!(".{}", method.name);
                let arguments = tupled(args.iter().map(Pretty::pretty));
                group(result + arguments)
            }
            Expr::Assign {
                ref to,
                op,
                ref from,
            } => {
                let op_string = op.map_or_else(|| "".to_owned(), |op| format!("{}", op));
                let start = to.pretty() + format!(" {}=", op_string);
                group(group(start) + spaceline() + from.pretty())
            }
            Expr::New { .. } => unimplemented!(),
            Expr::This => "this".into(),
            Expr::Super => "super".into(),
        }
    }
}

impl<T> PrettyWith<T> for Assignable {
    fn pretty_with(&self, _: &T) -> Doc {
        match *self {
            Assignable::Variable(ref ident, _) => ident.into(),
            Assignable::Field {
                ref this,
                ref class,
                ref field,
            } => {
                let result = if let Some(ref this) = *this {
                    this.pretty()
                } else {
                    class.0.to_owned().into()
                };
                result + breakline() + format!(".{}", field.name)
            }
            Assignable::ArrayAccess { .. } => unimplemented!(),
        }
    }
}

impl<T> PrettyWith<T> for Statement {
    fn pretty_with(&self, _: &T) -> Doc {
        match *self {
            Statement::Expr(ref e) => nest(4, e.pretty() + ";"),
            Statement::Block(ref block) => block.pretty(),
            Statement::If {
                ref cond,
                ref then,
                ref els,
            } => {
                doc("if (") + cond.pretty() + ") " + then.pretty()
                    + els.as_ref()
                        .map_or_else(empty, |e| doc(" else ") + e.pretty())
            }
            Statement::While {
                ref label,
                ref cond,
                ref body,
                do_while,
            } => {
                let while_part = doc("while (") + cond.pretty() + ")";
                let while_part = if let Some(ref label) = *label {
                    group(doc(label) + ':' + spaceline() + while_part)
                } else {
                    while_part
                };
                let (header, footer) = if do_while {
                    (doc("do "), while_part + ';')
                } else {
                    (while_part + ' ', empty())
                };
                header + body.pretty() + footer
            }
            Statement::For(..) => unimplemented!(),
            Statement::Break(ref label) => {
                doc("break") + label.as_ref().map_or_else(empty, |l| doc(" ") + l) + ";"
            }
            Statement::Continue(ref label) => {
                doc("continue") + label.as_ref().map_or_else(empty, |l| doc(" ") + l) + ";"
            }
            Statement::Return(ref val) => {
                doc("return") + val.as_ref().map_or_else(empty, |v| doc(" ") + v.pretty()) + ";"
            }
            Statement::ThisCall(ref args) => doc("this") + tupled(args.iter().map(Pretty::pretty)),
            Statement::SuperCall(ref args) => {
                doc("super") + tupled(args.iter().map(Pretty::pretty))
            }
            Statement::Throw(..) => unimplemented!(),
            Statement::Synchronized(..) => unimplemented!(),
            Statement::Try { .. } => unimplemented!(),
        }
    }
}

impl<T> PrettyWith<T> for Block {
    fn pretty_with(&self, _: &T) -> Doc {
        let declarations = &self.0;
        let separator = if declarations.is_empty() {
            empty()
        } else {
            newline()
        };
        let declarations = intersperse(declarations.iter().map(|d| d.pretty()), newline());
        let statements = &self.1;
        let statements = intersperse(statements.iter().map(|d| d.pretty()), newline());
        doc('{') + nest(4, newline() + declarations + separator + statements) + newline() + '}'
    }
}

impl<T> PrettyWith<T> for LocalDecl {
    fn pretty_with(&self, _: &T) -> Doc {
        let initializer = if let Some(ref value) = self.init {
            group(nest(4, doc(" =") + spaceline() + value.pretty()))
        } else {
            empty()
        };
        doc(&self.typ) + format!(" {}", self.ident) + initializer + ';'
    }
}
