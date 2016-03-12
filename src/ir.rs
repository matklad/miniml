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

    fn internal_name(&mut self) -> Name {
        return 1;
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
            Expr::Fun(ref fun) => fun.desugar(renamer),
            Expr::LetFun(ref let_fun) => let_fun.desugar(renamer),
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

impl Sugar for syntax::Fun {
    fn desugar<'e>(&'e self, renamer: &mut Renamer<'e>) -> Ir {
        Fun {
            fun_name: renamer.lookup(self.fun_name.as_ref()),
            arg_name: renamer.lookup(self.arg_name.as_ref()),
            body: self.body.desugar(renamer),
        }
        .into()
    }
}

impl Sugar for syntax::LetFun {
    fn desugar<'e>(&'e self, renamer: &mut Renamer<'e>) -> Ir {
        let fun = self.fun.desugar(renamer);
        let expr = self.body.desugar(renamer);
        Apply {
            fun: Fun {
                     fun_name: renamer.internal_name(),
                     arg_name: renamer.lookup(self.fun.fun_name.as_ref()),
                     body: expr,
                 }
                 .into(),
            arg: fun.into(),
        }
        .into()
    }
}
