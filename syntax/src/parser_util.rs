use ast::{Ident, Type, Expr, ArithBinOp, ArithOp, CmpBinOp, CmpOp, If, Apply, Fun, LetFun, LetRec};

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
        fun_name: name,
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

pub fn let_rec_expr(funs: Vec<Fun>, last_fun: Fun, body: Expr) -> Expr {
    let funs = {
        let mut funs = funs;
        funs.push(last_fun);
        funs
    };

    LetRec {
        funs: funs,
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
