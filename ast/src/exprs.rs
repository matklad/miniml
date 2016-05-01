use Type;
use Ident;
use std::fmt::{self, Write};


pub enum Expr {
    Var(Ident),
    Literal(Literal),
    ArithBinOp(Box<ArithBinOp>),
    CmpBinOp(Box<CmpBinOp>),
    If(Box<If>),
    Fun(Box<Fun>),
    LetFun(Box<LetFun>),
    LetRec(Box<LetRec>),
    Apply(Box<Apply>),
}

macro_rules! into_expr {
    ($id:ident) => {
        impl Into<Expr> for $id {
            fn into(self) -> Expr {
                Expr::$id(Box::new(self))
            }
        }
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Expr::*;
        match *self {
            Var(ref s) => f.write_str(s.as_ref()),
            Literal(ref l) => l.fmt(f),
            ArithBinOp(ref op) => op.fmt(f),
            CmpBinOp(ref op) => op.fmt(f),
            If(ref if_) => if_.fmt(f),
            Apply(ref apply) => apply.fmt(f),
            Fun(ref fun) => fun.fmt(f),
            LetFun(ref let_fun) => let_fun.fmt(f),
            LetRec(ref let_rec) => let_rec.fmt(f),
        }
    }
}

pub struct BinOp<T> {
    pub kind: T,
    pub lhs: Expr,
    pub rhs: Expr,
}

impl<T: fmt::Debug> fmt::Debug for BinOp<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:?} {:?} {:?})", self.kind, self.lhs, self.rhs)
    }
}

#[derive(Clone, Copy)]
pub enum ArithOp {
    Mul,
    Div,
    Add,
    Sub,
}

impl fmt::Debug for ArithOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ArithOp::*;
        f.write_char(match *self {
            Mul => '*',
            Div => '\\',
            Add => '+',
            Sub => '-',
        })
    }
}

pub type ArithBinOp = BinOp<ArithOp>;

into_expr!(ArithBinOp);

#[derive(Clone, Copy)]
pub enum CmpOp {
    Eq,
    Lt,
    Gt,
}

impl fmt::Debug for CmpOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::CmpOp::*;
        f.write_str(match *self {
            Eq => "==",
            Lt => "<",
            Gt => ">",
        })
    }
}

pub type CmpBinOp = BinOp<CmpOp>;

into_expr!(CmpBinOp);

pub struct If {
    pub cond: Expr,
    pub tru: Expr,
    pub fls: Expr,
}

into_expr!(If);

impl fmt::Debug for If {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(if {:?} {:?} {:?})", self.cond, self.tru, self.fls)
    }
}

pub struct Fun {
    pub fun_name: Ident,
    pub arg_name: Ident,
    pub arg_type: Type,
    pub fun_type: Type,
    pub body: Expr,
}

into_expr!(Fun);

impl fmt::Debug for Fun {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
        "(λ {} ({}: {:?}): {:?} {:?})",
        self.fun_name,
        self.arg_name,
        self.arg_type,
        self.fun_type,
        self.body)
    }
}

pub struct LetFun {
    pub fun: Fun,
    pub body: Expr,
}

into_expr!(LetFun);

impl fmt::Debug for LetFun {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
        "(let {} λ({}: {:?}): {:?} {:?} in {:?})",
        self.fun.fun_name,
        self.fun.arg_name,
        self.fun.arg_type,
        self.fun.fun_type,
        self.fun.body,
        self.body)
    }
}

pub struct LetRec {
    pub funs: Vec<Fun>,
    pub body: Expr,
}

into_expr!(LetRec);

impl fmt::Debug for LetRec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "(letrec ["));
        for fun in &self.funs {
            try!(write!(f, "{:?}", fun));
        }
        write!(f, "] in {:?})", self.body)
    }
}

pub struct Apply {
    pub fun: Expr,
    pub arg: Expr,
}

into_expr!(Apply);

impl fmt::Debug for Apply {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:?} {:?})", self.fun, self.arg)
    }
}

pub enum Literal {
    Number(i64),
    Bool(bool),
}

impl Into<Expr> for Literal {
    fn into(self) -> Expr {
        Expr::Literal(self)
    }
}

impl fmt::Debug for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Literal::Number(x) => x.fmt(f),
            Literal::Bool(b) => b.fmt(f),
        }
    }
}
