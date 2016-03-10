use Ident;
use super::exprs::{Expr, ArithBinOp, ArithOp, CmpBinOp, CmpOp, If, Apply, Fun, LetFun};
use super::types::Type;

pub fn arith_op(l: Expr, op: ArithOp, r: Expr) -> Expr {
    ArithBinOp {
        kind: op,
        lhs: l,
        rhs: r,
    }
    .into()
}

pub fn cmp_op(l: Expr, op: CmpOp, r: Expr) -> Expr {
    CmpBinOp {
        kind: op,
        lhs: l,
        rhs: r,
    }
    .into()
}

pub fn if_expr(cond: Expr, tru: Expr, fls: Expr) -> Expr {
    If {
        cond: cond,
        tru: tru,
        fls: fls,
    }
    .into()
}

pub fn fun(name: Ident, arg_name: Ident, arg_type: Type, fun_type: Type, body: Expr) -> Fun {
    Fun {
        name: name,
        arg_name: arg_name,
        arg_type: arg_type,
        fun_type: fun_type,
        body: body,
    }
}

pub fn let_fun_expr(fun: Fun, body: Expr) -> Expr {
    LetFun {
        fun: fun,
        body: body,
    }.into()
}

pub fn application(fun: Expr, arg: Expr) -> Expr {
    Apply {
        fun: fun,
        arg: arg,
    }
    .into()
}
