use super::exprs::{Expr, ArithBinOp, ArithOp, CmpBinOp, CmpOp, If, Apply};

pub fn arith_op(l: Box<Expr>, op: ArithOp, r: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::ArithBinOp(ArithBinOp {
        kind: op,
        lhs: l,
        rhs: r,
    }))
}

pub fn cmp_op(l: Box<Expr>, op: CmpOp, r: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::CmpBinOp(CmpBinOp {
        kind: op,
        lhs: l,
        rhs: r,
    }))
}

pub fn if_expr(cond: Box<Expr>, tru: Box<Expr>, fls: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::If(If {
        cond: cond,
        tru: tru,
        fls: fls,
    }))
}

pub fn application(fun: Box<Expr>, arg: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Apply(Apply {
        fun: fun,
        arg: arg,
    }))
}
