use super::{Expr, ArithBinOp, ArithOp, CmpBinOp, CmpOp};

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
