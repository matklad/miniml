use super::exprs::{Expr, ArithBinOp, ArithOp, CmpBinOp, CmpOp, If, Apply, Fun};
use super::types::Type;

pub fn arith_op(l: Expr, op: ArithOp, r: Expr) -> Expr {
    Expr::ArithBinOp(Box::new(ArithBinOp {
        kind: op,
        lhs: l,
        rhs: r,
    }))
}

pub fn cmp_op(l: Expr, op: CmpOp, r: Expr) -> Expr {
    Expr::CmpBinOp(Box::new(CmpBinOp {
        kind: op,
        lhs: l,
        rhs: r,
    }))
}

pub fn if_expr(cond: Expr, tru: Expr, fls: Expr) -> Expr {
    Expr::If(Box::new(If {
        cond: cond,
        tru: tru,
        fls: fls,
    }))
}

pub fn fun_expr(name: String,
                arg_name: String,
                arg_type: Type,
                fun_type: Type,
                body: Expr) -> Expr {
    Expr::Fun(Box::new(Fun {
        name: name,
        arg_name: arg_name,
        arg_type: arg_type,
        fun_type: fun_type,
        body: body,
    }))
}

pub fn application(fun: Expr, arg: Expr) -> Expr {
    Expr::Apply(Box::new(Apply {
        fun: fun,
        arg: arg,
    }))
}
