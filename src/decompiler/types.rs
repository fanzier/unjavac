use disassembler::types::*;
use std::fmt::*;

pub type Ident = String;

#[derive(Clone, Debug, Hash)]
pub enum Expr {
    Literal(Literal),
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
    New {
        class: Type,
        args: Vec<Expr>,
    },
    This,
    Super,
}

pub trait Visitor {
    fn visit_block(&mut self, block: &mut Block) {
        walk_block(self, block);
    }
    fn visit_statement(&mut self, stmt: &mut Statement) {
        walk_statement(self, stmt);
    }
    fn visit_expr(&mut self, expr: &mut Expr) {
        walk_expr(self, expr);
    }
    fn visit_assignable(&mut self, assignable: &mut Assignable) {
        walk_assignable(self, assignable);
    }
}

pub fn walk_block<V: Visitor + ?Sized>(
    visitor: &mut V,
    &mut Block(ref mut decls, ref mut stmts): &mut Block,
) {
    for decl in decls {
        if let LocalDecl {
            init: Some(ref mut expr),
            ..
        } = *decl
        {
            visitor.visit_expr(expr);
        }
    }
    for stmt in stmts {
        visitor.visit_statement(stmt);
    }
}

pub fn walk_statement<V: Visitor + ?Sized>(visitor: &mut V, stmt: &mut Statement) {
    match *stmt {
        Statement::Nop => (),
        Statement::Expr(ref mut expr) => visitor.visit_expr(expr),
        Statement::Block(ref mut block) => visitor.visit_block(block),
        Statement::If {
            ref mut cond,
            ref mut then,
            ref mut els,
        } => {
            visitor.visit_expr(cond);
            visitor.visit_block(then);
            els.as_mut().map(|els| visitor.visit_block(els));
        }
        Statement::While {
            ref mut cond,
            ref mut body,
            ..
        } => {
            visitor.visit_expr(cond);
            visitor.visit_block(body);
        }
        Statement::For(.., ref mut control, ref mut body) => {
            match *control.as_mut() {
                ForControl::Iteration {
                    ref mut container, ..
                } => visitor.visit_expr(container),
                ForControl::General {
                    ref mut cond,
                    ref mut update,
                    ..
                } => {
                    visitor.visit_expr(cond);
                    visitor.visit_expr(update);
                }
            }
            visitor.visit_block(body);
        }
        Statement::Break(..) | Statement::Continue(..) => (),
        Statement::Return(ref mut expr) => {
            expr.as_mut().map(|expr| visitor.visit_expr(expr));
        }
        Statement::ThisCall(ref mut args) | Statement::SuperCall(ref mut args) => {
            for expr in args {
                visitor.visit_expr(expr);
            }
        }
        Statement::Throw(..) => unimplemented!(),
        Statement::Synchronized(..) => unimplemented!(),
        Statement::Try { .. } => unimplemented!(),
    }
}

pub fn walk_expr<V: Visitor + ?Sized>(visitor: &mut V, expr: &mut Expr) {
    match *expr {
        Expr::Literal(..) => (),
        Expr::Assignable(ref mut assignable) => visitor.visit_assignable(assignable),
        Expr::UnaryOp(_, ref mut expr) => visitor.visit_expr(expr.as_mut()),
        Expr::BinaryOp(_, ref mut e1, ref mut e2) => {
            visitor.visit_expr(e1.as_mut());
            visitor.visit_expr(e2.as_mut())
        }
        Expr::IfThenElse {
            ref mut cond,
            ref mut then,
            ref mut els,
        } => {
            visitor.visit_expr(cond);
            visitor.visit_expr(then);
            visitor.visit_expr(els);
        }
        Expr::Invoke(ref mut this, .., ref mut exprs) => {
            this.as_mut().map(|expr| visitor.visit_expr(expr));
            for expr in exprs {
                visitor.visit_expr(expr);
            }
        }
        Expr::Assign {
            ref mut to,
            ref mut from,
            ..
        } => {
            visitor.visit_assignable(to.as_mut());
            visitor.visit_expr(from.as_mut());
        }
        Expr::New { ref mut args, .. } => {
            for expr in args {
                visitor.visit_expr(expr)
            }
        }
        Expr::This => (),
        Expr::Super => (),
    }
}

pub fn walk_assignable<V: Visitor + ?Sized>(visitor: &mut V, assignable: &mut Assignable) {
    match *assignable {
        Assignable::Variable(..) => (),
        Assignable::Field { ref mut this, .. } => if let Some(ref mut expr) = this {
            visitor.visit_expr(expr.as_mut());
        },
        Assignable::ArrayAccess {
            ref mut array,
            ref mut index,
        } => {
            visitor.visit_expr(array.as_mut());
            visitor.visit_expr(index.as_mut());
        }
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
    ArrayAccess {
        array: Box<Expr>,
        index: Box<Expr>,
    },
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

pub fn stmt_expr(e: Expr) -> Statement {
    Statement::Expr(e)
}

#[derive(Clone, Debug, Hash)]
pub enum Statement {
    Nop,
    Expr(Expr),
    Block(Block),
    If {
        cond: Expr,
        then: Block,
        els: Option<Block>,
    },
    While {
        label: Option<Ident>,
        cond: Expr,
        body: Block,
        do_while: bool,
    },
    For(Option<Ident>, Box<ForControl>, Block),
    Break(Option<Ident>),
    Continue(Option<Ident>),
    Return(Option<Expr>),
    ThisCall(Vec<Expr>),
    SuperCall(Vec<Expr>),
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

impl Default for Block {
    fn default() -> Block {
        Block(vec![], vec![])
    }
}

#[derive(Clone, Debug, Hash)]
pub struct LocalDecl {
    pub ident: Ident,
    pub typ: Type,
    pub init: Option<Expr>,
}

#[derive(Clone, Debug, Hash)]
pub enum ForControl {
    Iteration {
        elem: LocalDecl,
        container: Expr,
    },
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
