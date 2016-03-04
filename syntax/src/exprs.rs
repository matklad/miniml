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
    Apply(Box<Apply>),
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

pub enum ArithOp { Mul, Div, Add, Sub }

impl fmt::Debug for ArithOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ArithOp::*;
        f.write_char(match *self {
            Mul => '*', Div => '\\',
            Add => '+', Sub => '-',
         })
    }
}

pub type ArithBinOp = BinOp<ArithOp>;

impl Into<Expr> for ArithBinOp {
    fn into(self) -> Expr { Expr::ArithBinOp(Box::new(self)) }
}

pub enum CmpOp { Eq, Lt, Gt }

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

impl Into<Expr> for CmpBinOp {
    fn into(self) -> Expr { Expr::CmpBinOp(Box::new(self)) }
}

pub struct If {
    pub cond: Expr,
    pub tru: Expr,
    pub fls: Expr,
}

impl fmt::Debug for If {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(if {:?} {:?} {:?})", self.cond, self.tru, self.fls)
    }
}

impl Into<Expr> for If {
    fn into(self) -> Expr { Expr::If(Box::new(self)) }
}

pub struct Fun {
    pub name: Ident,
    pub arg_name: Ident,
    pub arg_type: Type,
    pub fun_type: Type,
    pub body: Expr,
}

impl Into<Expr> for Fun {
    fn into(self) -> Expr { Expr::Fun(Box::new(self)) }
}

impl fmt::Debug for Fun {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(λ {} ({}: {:?}): {:?} {:?})",
               self.name, self.arg_name, self.arg_type, self.fun_type, self.body)
    }
}

pub struct Apply {
    pub fun: Expr,
    pub arg: Expr,
}

impl Into<Expr> for Apply {
    fn into(self) -> Expr { Expr::Apply(Box::new(self)) }
}

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
    fn into(self) -> Expr { Expr::Literal(self) }
}

impl fmt::Debug for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Literal::Number(x) => x.fmt(f),
            Literal::Bool(b) => b.fmt(f),
        }
    }
}
