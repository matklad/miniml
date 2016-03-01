use super::exprs::{Expr, ArithBinOp, ArithOp, CmpBinOp, CmpOp, If, Apply, Fun};
use super::types::Type;

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

pub fn fun_expr(name: String,
                arg_name: String,
                arg_type: Box<Type>,
                fun_type: Box<Type>,
                body: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Fun(Fun {
        name: name,
        arg_name: arg_name,
        arg_type: arg_type,
        fun_type: fun_type,
        body: body,
    }))
}

pub fn application(fun: Box<Expr>, arg: Box<Expr>) -> Box<Expr> {
    Box::new(Expr::Apply(Apply {
        fun: fun,
        arg: arg,
    }))
}
