use std::collections::HashMap;
use syntax::{self, Expr};

pub type Name = usize;

pub enum Ir {
    Var(Name),
    IntLiteral(i64),
    BoolLiteral(bool),
    BinOp(Box<BinOp>),
    If(Box<If>),
    Fun(Box<Fun>),
    LetFun(Box<LetFun>),
    Apply(Box<Apply>),
}

pub fn desugar(expr: &Expr) -> Ir {
    let mut renamer = Renamer::empty();
    expr.desugar(&mut renamer)
}

macro_rules! into_ir {
    ($id:ident) => {
        impl Into<Ir> for $id {
            fn into(self) -> Ir {
                Ir::$id(Box::new(self))
            }
        }
    }
}

pub struct BinOp {
    pub lhs: Ir,
    pub rhs: Ir,
    pub kind: BinOpKind,
}

into_ir!(BinOp);

pub enum BinOpKind {
    Add,
    Sub,
    Div,
    Mul,
    Lt,
    Eq,
    Gt,
}

pub struct If {
    pub cond: Ir,
    pub tru: Ir,
    pub fls: Ir,
}

into_ir!(If);

pub struct Fun {
    pub fun_name: Name,
    pub arg_name: Name,
    pub body: Ir,
}

into_ir!(Fun);

pub struct LetFun {
    pub fun_name: Name,
    pub arg_name: Name,
    pub fun_body: Ir,
    pub expr: Ir,
}

into_ir!(LetFun);

pub struct Apply {
    pub fun: Ir,
    pub arg: Ir,
}

into_ir!(Apply);

struct Renamer<'a> {
    names: HashMap<&'a str, Name>,
}

impl<'a> Renamer<'a> {
    fn empty() -> Renamer<'static> {
        Renamer { names: HashMap::new() }
    }

    fn lookup(&mut self, name: &'a str) -> Name {
        if !self.names.contains_key(name) {
            let new_id = self.names.len();
            self.names.insert(name, new_id);
        }
        self.names[name] * 2
    }
}

trait Sugar {
    fn desugar<'e>(&'e self, &mut Renamer<'e>) -> Ir;
}

impl Sugar for Expr {
    fn desugar<'e>(&'e self, renamer: &mut Renamer<'e>) -> Ir {
        match *self {
            Expr::Var(ref v) => Ir::Var(renamer.lookup(v.as_ref())),
            Expr::Literal(syntax::Literal::Number(n)) => Ir::IntLiteral(n),
            Expr::Literal(syntax::Literal::Bool(b)) => Ir::BoolLiteral(b),
            Expr::ArithBinOp(ref op) => op.desugar(renamer),
            Expr::CmpBinOp(ref op) => op.desugar(renamer),
            Expr::If(ref if_) => {
                If {
                    cond: if_.cond.desugar(renamer),
                    tru: if_.tru.desugar(renamer),
                    fls: if_.fls.desugar(renamer),
                }
                .into()
            }
            Expr::Fun(ref fun) => {
                Fun {
                    fun_name: renamer.lookup(fun.name.as_ref()),
                    arg_name: renamer.lookup(fun.arg_name.as_ref()),
                    body: fun.body.desugar(renamer),
                }
                .into()
            }
            Expr::LetFun(ref let_fun) => {
                LetFun {
                    fun_name: renamer.lookup(let_fun.fun.name.as_ref()),
                    arg_name: renamer.lookup(let_fun.fun.arg_name.as_ref()),
                    fun_body: let_fun.fun.body.desugar(renamer),
                    expr: let_fun.body.desugar(renamer),
                }
                .into()
            }
            Expr::Apply(ref apply) => {
                Apply {
                    fun: apply.fun.desugar(renamer),
                    arg: apply.arg.desugar(renamer),
                }
                .into()
            }
        }
    }
}

impl From<syntax::ArithOp> for BinOpKind {
    fn from(op: syntax::ArithOp) -> Self {
        match op {
            syntax::ArithOp::Add => BinOpKind::Add,
            syntax::ArithOp::Sub => BinOpKind::Sub,
            syntax::ArithOp::Mul => BinOpKind::Mul,
            syntax::ArithOp::Div => BinOpKind::Div,
        }
    }
}

impl From<syntax::CmpOp> for BinOpKind {
    fn from(op: syntax::CmpOp) -> Self {
        match op {
            syntax::CmpOp::Lt => BinOpKind::Lt,
            syntax::CmpOp::Eq => BinOpKind::Eq,
            syntax::CmpOp::Gt => BinOpKind::Gt,
        }
    }
}

impl<OP> Sugar for syntax::BinOp<OP>
    where BinOpKind: From<OP>,
          OP: Copy
{
    fn desugar<'e>(&'e self, renamer: &mut Renamer<'e>) -> Ir {
        BinOp {
            lhs: self.lhs.desugar(renamer),
            rhs: self.rhs.desugar(renamer),
            kind: BinOpKind::from(self.kind),
        }
        .into()
    }
}
