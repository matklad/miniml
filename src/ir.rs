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
            Expr::LetRec(ref let_rec) => let_rec.desugar(renamer),
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
        desugar_fun(self, renamer).into()
    }
}

fn desugar_fun<'e>(fun: &'e syntax::Fun, renamer: &mut Renamer<'e>) -> Fun {
    Fun {
        fun_name: renamer.lookup(fun.fun_name.as_ref()),
        arg_name: renamer.lookup(fun.arg_name.as_ref()),
        body: fun.body.desugar(renamer),
    }
}

impl Sugar for syntax::LetFun {
    fn desugar<'e>(&'e self, renamer: &mut Renamer<'e>) -> Ir {
        let fun = self.fun.desugar(renamer);
        let expr = self.body.desugar(renamer);
        Apply {
            fun: Fun {
                     fun_name: 1,
                     arg_name: renamer.lookup(self.fun.fun_name.as_ref()),
                     body: expr,
                 }
                 .into(),
            arg: fun.into(),
        }
        .into()
    }
}

impl Sugar for syntax::LetRec {
    // See tests `mutual_recursion3` for an example of transform.
    // On a high level, we convert a set of mutually recursive functions into a single function of
    // two arguments, the first of which is a tag
    fn desugar<'e>(&'e self, renamer: &mut Renamer<'e>) -> Ir {
        let funs = self.funs.iter().map(|fun| desugar_fun(fun, renamer)).collect::<Vec<_>>();
        let fun_names = funs.iter().map(|fun| fun.fun_name).collect::<Vec<_>>();

        let dispatch_arg = 5;
        let dispatch_if = {
            let mut result = undefined();
            for (i, fun) in funs.into_iter().enumerate() {
                let my_tag = i as i64;
                let dispatch_arg = Ir::Var(dispatch_arg);
                result = if_eq(dispatch_arg,
                               Ir::IntLiteral(my_tag),
                               fun_wrapper(my_tag, fun, &fun_names),
                               result)
            }
            result
        };
        let anon_name = 1;
        let dispatch_name = 3;
        let dispatch_fun: Ir = Fun {
                                   fun_name: dispatch_name,
                                   arg_name: dispatch_arg,
                                   body: dispatch_if,
                               }
                               .into();

        let mut result = self.body.desugar(renamer);
        for (i, name) in fun_names.into_iter().enumerate() {
            let f: Ir = Fun {
                            fun_name: anon_name,
                            arg_name: name,
                            body: result,
                        }
                        .into();
            result = f.apply(Ir::Var(dispatch_name).apply(Ir::IntLiteral(i as i64)))
        }

        let f: Ir = Fun {
                        fun_name: anon_name,
                        arg_name: dispatch_name,
                        body: result,
                    }
                    .into();
        f.apply(dispatch_fun)
    }
}

fn fun_wrapper(my_tag: i64, fun: Fun, fun_names: &[Name]) -> Ir {

    let mut bindins = vec![];
    let dispatch_name = 3;
    for (i, &name) in fun_names.iter().enumerate() {
        let fun_tag = i as i64;
        if fun_tag == my_tag {
            continue;
        }
        let x = 1;
        bindins.push(Fun {
            fun_name: name,
            arg_name: x,
            body: Ir::Var(dispatch_name)
                      .apply(Ir::IntLiteral(fun_tag))
                      .apply(Ir::Var(x)),
        })
    }

    Fun {
        fun_name: fun.fun_name,
        arg_name: fun.arg_name,
        body: lets(bindins, fun.body),
    }
    .into()
}

fn if_eq(lhs: Ir, rhs: Ir, tru: Ir, fls: Ir) -> Ir {
    If {
        cond: BinOp {
                  lhs: lhs,
                  rhs: rhs,
                  kind: BinOpKind::Eq,
              }
              .into(),
        tru: tru,
        fls: fls,
    }
    .into()
}

fn lets(mut bindings: Vec<Fun>, body: Ir) -> Ir {
    if let Some(head) = bindings.pop() {
        lets(bindings, let_(head, body))
    } else {
        body
    }
}

fn let_(fun: Fun, body: Ir) -> Ir {
    Apply {
        fun: Fun {
                 fun_name: 1,
                 arg_name: fun.fun_name,
                 body: body,
             }
             .into(),
        arg: fun.into(),
    }
    .into()

}

fn undefined() -> Ir {
    BinOp {
        lhs: Ir::IntLiteral(0),
        rhs: Ir::IntLiteral(0),
        kind: BinOpKind::Div,
    }
    .into()
}

impl Ir {
    fn apply<I: Into<Ir>>(self, arg: I) -> Ir {
        Apply {
            fun: self,
            arg: arg.into(),
        }
        .into()
    }
}
