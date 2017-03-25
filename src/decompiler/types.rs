use disassembler::compilation_unit::*;
use std::fmt::*;

type Ident = String;

pub fn rec_expr(e: Expr<RecExpr>) -> RecExpr {
    RecExpr(Box::new(e))
}

#[derive(Clone, Debug, Hash)]
pub struct RecExpr(pub Box<Expr<RecExpr>>);

impl RecExpr {
    pub fn inner(&self) -> &Expr<RecExpr> {
        &self.0
    }
}

#[derive(Clone, Debug, Hash)]
pub enum Expr<E> {
    Literal(Literal),
    Assignable(Box<Assignable>),
    UnaryOp(UnOp, E),
    BinaryOp(BinOp, E, E),
    IfThenElse { cond: E, then: E, els: E },
    Invoke(Option<E>, MethodRef, ClassRef, Vec<E>),
    Assign {
        to: Box<Assignable>,
        op: Option<BinOp>,
        from: E,
    },
    New { class: Type, args: Vec<E> },
    This,
    Super,
    // TODO this(...), super(...)
}

impl Display for RecExpr {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

pub fn mk_variable(id: Ident) -> RecExpr {
    rec_expr(Expr::Assignable(Box::new(Assignable::Variable(id, 0))))
}

#[derive(Clone, Debug, Hash)]
pub enum Assignable {
    Variable(Ident, usize),
    Field {
        this: Option<RecExpr>,
        class: ClassRef,
        field: FieldRef,
    },
    ArrayAccess { array: RecExpr, index: RecExpr },
}

#[derive(Copy, Clone, Debug, Hash)]
pub enum UnOp {
    Neg,
    BitNot,
    LogNot,
}

impl Display for UnOp {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let symbol = match *self {
            UnOp::Neg => "-",
            UnOp::BitNot => "~",
            UnOp::LogNot => "!",
        };
        write!(f, "{}", symbol)
    }
}

#[derive(Copy, Clone, Debug, Hash)]
pub enum BinOp {
    Cmp(Ordering),
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Shl,
    Shr,
    Ushr,
    BitAnd,
    BitOr,
    BitXor,
    LogAnd,
    LogOr,
}

impl Display for BinOp {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let symbol = match *self {
            BinOp::Cmp(ord) => ord.to_str(),
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::Shl => "<<",
            BinOp::Shr => ">>",
            BinOp::Ushr => ">>>",
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::BitXor => "^",
            BinOp::LogAnd => "&&",
            BinOp::LogOr => "||",
        };
        write!(f, "{}", symbol)
    }
}

pub fn stmt_expr(e: Expr<RecExpr>) -> Statement {
    Statement::Expr(rec_expr(e))
}

#[derive(Clone, Debug, Hash)]
pub enum Statement {
    Expr(RecExpr),
    Block(Block),
    If {
        cond: RecExpr,
        then: Block,
        els: Option<Block>,
    },
    While {
        cond: RecExpr,
        body: Block,
        do_while: bool,
    },
    For(Box<ForControl>),
    Label { label: Ident, stmt: Box<Statement> },
    Break(Option<Ident>),
    Continue(Option<Ident>),
    Return(Option<RecExpr>),
    Throw(RecExpr),
    Synchronized(RecExpr, Block),
    Try {
        resources: Vec<LocalDecl>,
        block: Block,
        catches: Vec<Catch>,
        finally: Block,
    }, // TODO: assert, switch
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, Hash)]
pub struct Block(pub Vec<LocalDecl>, pub Vec<Statement>);

#[derive(Clone, Debug, Hash)]
pub struct LocalDecl {
    pub ident: Ident,
    pub typ: Type,
    pub init: Option<RecExpr>,
}

#[derive(Clone, Debug, Hash)]
pub enum ForControl {
    Iteration { elem: LocalDecl, container: RecExpr },
    General {
        init: LocalDecl,
        cond: RecExpr,
        update: RecExpr,
    },
}

#[derive(Clone, Debug, Hash)]
pub struct Capsule<C> {
    pub modifiers: Vec<Modifier>,
    pub name: Ident,
    pub decls: Vec<ClassDecl<C>>,
}

#[derive(Clone, Debug, Hash)]
pub enum ClassDecl<C> {
    // TODO: InnerClass(Capsule<C>),
    Field(FieldDecl),
    Method(MethodDecl<C>),
}

#[derive(Clone, Debug, Hash)]
pub enum Catch {
    // TODO
}


#[derive(Clone, Debug, Hash)]
pub struct FieldDecl {
    // TODO
}

#[derive(Clone, Debug, Hash)]
pub struct MethodDecl<C> {
    pub modifiers: Vec<Modifier>,
    pub name: Ident,
    pub signature: Signature,
    pub code: C,
}
