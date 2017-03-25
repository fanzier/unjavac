use pretty::*;
use super::types::*;
use std::cmp::Ordering;

trait HasPrecedence {
    fn precedence(&self) -> Precedence;
}

#[derive(Copy, Clone, Debug, Hash, PartialEq)]
enum Precedence {
    Least = 0,
    Assign = 10, // =, +=, -=, ...
    Ternary = 20, // ? :
    LogOr = 30, // ||
    LogAnd = 31, // &&
    Cmp = 40, // ==, !=, <=, >=, >, <, instanceof
    BitOp = 50, // &, |, ^, >>, >>>, <<
    Add = 60, // +, -
    Mul = 70, // *, /, %
    Unary = 80, // -, +, !, ~, ++, --
    Access = 90, // (), [], .
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
    where T: HasPrecedence + Pretty
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
            Assignable::Field { .. } |
            Assignable::ArrayAccess { .. } => Precedence::Access,
        }
    }
}

impl<E> HasPrecedence for Expr<E> {
    fn precedence(&self) -> Precedence {
        match *self {
            Expr::Assignable(ref v) => v.precedence(),
            Expr::UnaryOp(..) => Precedence::Unary,
            Expr::BinaryOp(op, _, _) => {
                match op {
                    BinOp::Cmp(_) => Precedence::Cmp,
                    BinOp::Add | BinOp::Sub => Precedence::Add,
                    BinOp::Mul | BinOp::Div | BinOp::Rem => Precedence::Mul,
                    BinOp::LogAnd => Precedence::LogAnd,
                    BinOp::LogOr => Precedence::LogOr,
                    BinOp::Shl | BinOp::Shr | BinOp::Ushr | BinOp::BitAnd | BinOp::BitOr |
                    BinOp::BitXor => Precedence::BitOp,
                }
            }
            Expr::IfThenElse { .. } => Precedence::Ternary,
            Expr::Invoke(..) |
            Expr::Assign { .. } => Precedence::Assign,
            Expr::Literal(_) | Expr::New { .. } | Expr::This | Expr::Super => Precedence::Tightest,
        }
    }
}

impl<T> PrettyWith<T> for RecExpr {
    fn pretty_with(&self, _: &T) -> Doc {
        self.inner().pretty()
    }
}

impl<T> PrettyWith<T> for Expr<RecExpr> {
    fn pretty_with(&self, _: &T) -> Doc {
        match *self {
            Expr::Literal(ref literal) => format!("{}", literal).into(),
            Expr::Assignable(ref v) => v.pretty(),
            Expr::UnaryOp(op, ref e) => {
                Doc::from(op) + parens_if(e.inner(), self.precedence(), true)
            }
            Expr::BinaryOp(op, ref e1, ref e2) => {
                (parens_if(e1.inner(), self.precedence(), true) + spaceline() +
                 group(format!("{} ", op).into()) +
                 parens_if(e2.inner(), self.precedence(), true))
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
            Expr::Assign { ref to, op, ref from } => {
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
            Assignable::Field { ref this, ref class, ref field } => {
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
        let result = match *self {
            Statement::Expr(ref e) => e.pretty() + ";",
            Statement::Return(ref val) => {
                doc("return") + val.as_ref().map_or(empty(), |v| doc(" ") + v.pretty()) + ";"
            }
            _ => unimplemented!(),
        };
        result.nest(4)
    }
}
