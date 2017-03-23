use disassembler::compilation_unit::*;
use std::fmt::*;

type Ident = String;

#[derive(Clone, Debug, Hash)]
pub enum Expr {
    Literal(JavaConstant),
    Assignable(Box<Assignable>),
    UnaryOp(UnOp, Box<Expr>),
    BinaryOp(BinOp, Box<Expr>, Box<Expr>),
    IfThenElse {
        cond: Box<Expr>,
        then: Box<Expr>,
        els: Box<Expr>,
    },
    Invoke(Option<Box<Expr>>, MethodRef, ClassRef, Vec<Expr>),
    Assign {
        to: Box<Assignable>,
        op: Option<BinOp>,
        from: Box<Expr>,
    },
    New { class: Type, args: Vec<Expr> },
    This,
    Super,
    // TODO this(...), super(...)
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

pub fn mk_variable(id: Ident) -> Expr {
    Expr::Assignable(Box::new(Assignable::Variable(id, 0)))
}

#[derive(Clone, Debug, Hash)]
pub enum Assignable {
    Variable(Ident, usize),
    Field {
        this: Option<Box<Expr>>,
        class: ClassRef,
        field: FieldRef,
    },
    ArrayAccess { array: Box<Expr>, index: Box<Expr> },
}

#[derive(Copy, Clone, Debug, Hash)]
pub enum UnOp {
    Neg,
    Not,
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
    And,
    Or,
    Xor,
}

#[derive(Clone, Debug, Hash)]
pub enum Statement {
    Expr(Expr),
    Block(Block),
    If {
        cond: Expr,
        then: Block,
        els: Option<Block>,
    },
    While {
        cond: Expr,
        body: Block,
        do_while: bool,
    },
    For(Box<ForControl>),
    Label { label: Ident, stmt: Box<Statement> },
    Break(Option<Ident>),
    Continue(Option<Ident>),
    Return(Option<Expr>),
    Throw(Expr),
    Synchronized(Expr, Block),
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
    pub init: Option<Expr>,
}

#[derive(Clone, Debug, Hash)]
pub enum ForControl {
    Iteration { elem: LocalDecl, container: Expr },
    General {
        init: LocalDecl,
        cond: Expr,
        update: Expr,
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
