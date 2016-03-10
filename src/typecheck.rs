use std::rc::Rc;
use std::fmt;

use syntax::{self, Expr, Literal, ArithBinOp, CmpBinOp, If, Fun, LetFun, Apply};
use context::{Context, StackContext};

pub type Result = ::std::result::Result<Type, TypeError>;

#[derive(Debug)]
pub struct TypeError {
    pub message: String,
}

#[derive(PartialEq, Eq, Clone)]
pub enum Type {
    Int,
    Bool,
    Arrow(Rc<Type>, Rc<Type>),
}

use self::Type::*;

impl Type {
    fn maps_to(self, other: Type) -> Type {
        Arrow(Rc::new(self), Rc::new(other))
    }
}

trait IntoType {
    fn as_type(&self) -> Type;
}

impl IntoType for syntax::Type {
    fn as_type(&self) -> Type {
        match *self {
            syntax::Type::Int => Int,
            syntax::Type::Bool => Bool,
            syntax::Type::Arrow(ref l, ref r) => Arrow(Rc::new(l.as_type()), Rc::new(r.as_type())),
        }
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Int => f.write_str("int"),
            Bool => f.write_str("bool"),
            Arrow(ref l, ref r) => {
                match **l {
                    Arrow(..) => write!(f, "({:?}) -> {:?}", l, r),
                    _ => write!(f, "{:?} -> {:?}", l, r),
                }
            }
        }
    }
}

pub fn typecheck(expr: &Expr) -> Result {
    let mut ctx = StackContext::new();
    expr.check(&mut ctx)
}

macro_rules! bail {
    ($msg:expr) => { bail!($e, $msg,) };

    ($msg:expr, $($farg:expr),*) => {
        return Err(TypeError {
            message: format!($msg $(, $farg)*),
        })
    };
}

fn expect<'c, C: Context<'c, Type>>(expr: &'c Expr, type_: Type, ctx: &mut C) -> Result {
    let t = try!(expr.check(ctx));
    if t != type_ {
        bail!("Expected {:?}, got {:?}", type_, t);
    }
    Ok(type_)
}

trait Typecheck {
    fn check<'c, C: Context<'c, Type>>(&'c self, ctx: &mut C) -> Result;
}

impl Typecheck for Expr {
    fn check<'c, C: Context<'c, Type>>(&'c self, ctx: &mut C) -> Result {
        use syntax::Expr::*;
        match *self {
            Var(ref ident) => {
                ctx.lookup(ident)
                   .cloned()
                   .ok_or(TypeError { message: format!("Unbound variable: {}", ident) })
            }
            Literal(ref l) => l.check(ctx),
            ArithBinOp(ref op) => op.check(ctx),
            CmpBinOp(ref op) => op.check(ctx),
            If(ref if_) => if_.check(ctx),
            Fun(ref fun) => fun.check(ctx),
            LetFun(ref let_fun) => let_fun.check(ctx),
            Apply(ref apply) => apply.check(ctx),
        }
    }
}

impl Typecheck for Literal {
    fn check<'c, C: Context<'c, Type>>(&'c self, _: &mut C) -> Result {
        let t = match *self {
            Literal::Number(_) => Int,
            Literal::Bool(_) => Bool,
        };
        Ok(t)
    }
}

impl Typecheck for ArithBinOp {
    fn check<'c, C: Context<'c, Type>>(&'c self, ctx: &mut C) -> Result {
        try!(expect(&self.lhs, Int, ctx));
        try!(expect(&self.rhs, Int, ctx));
        Ok(Int)
    }
}

impl Typecheck for CmpBinOp {
    fn check<'c, C: Context<'c, Type>>(&'c self, ctx: &mut C) -> Result {
        try!(expect(&self.lhs, Int, ctx));
        try!(expect(&self.rhs, Int, ctx));
        Ok(Bool)
    }
}

impl Typecheck for If {
    fn check<'c, C: Context<'c, Type>>(&'c self, ctx: &mut C) -> Result {
        try!(expect(&self.cond, Bool, ctx));
        let t1 = try!(self.tru.check(ctx));
        let t2 = try!(self.fls.check(ctx));
        if t1 != t2 {
            bail!("Arms of an if have different types: {:?} {:?}", t1, t2);
        }
        Ok(t1)
    }
}

impl Typecheck for Fun {
    fn check<'c, C: Context<'c, Type>>(&'c self, ctx: &mut C) -> Result {
        let arg_type = self.arg_type.as_type();
        let ret_type = self.fun_type.as_type();
        let result = arg_type.clone().maps_to(ret_type.clone());
        ctx.push(&self.arg_name, arg_type.clone());
        ctx.push(&self.name, result.clone());
        try!(expect(&self.body, ret_type.clone(), ctx));
        ctx.pop();
        ctx.pop();
        Ok(result)
    }
}

impl Typecheck for LetFun {
    fn check<'c, C: Context<'c, Type>>(&'c self, ctx: &mut C) -> Result {
        let fun_type = try!(self.fun.check(ctx));
        ctx.push(&self.fun.name, fun_type);
        let result = try!(self.body.check(ctx));
        ctx.pop();
        Ok(result)
    }
}

impl Typecheck for Apply {
    fn check<'c, C: Context<'c, Type>>(&'c self, ctx: &mut C) -> Result {
        match try!(self.fun.check(ctx)) {
            Type::Arrow(arg, ret) => {
                try!(expect(&self.arg, arg.as_ref().clone(), ctx));
                Ok(ret.as_ref().clone())
            }
            _ => return bail!("Not a function {:?}", self.fun),
        }
    }
}

#[cfg(test)]
mod tests {
    use syntax::Expr;
    use super::*;
    use super::Type::*;

    fn parse(expr: &str) -> Expr {
        ::syntax::parse(expr).expect(&format!("Failed to parse {}", expr))
    }

    fn assert_valid(expr: &str, type_: Type) {
        let expr = parse(expr);
        match typecheck(&expr) {
            Ok(t) => {
                assert!(t == type_,
                        "Wrong type for {:?}.\nExpected {:?}, got {:?}",
                        expr,
                        type_,
                        t)
            }
            Err(e) => assert!(false, "Failed to typecheck {:?}:\n {:?}", expr, e),
        }
    }

    fn assert_fails(expr: &str) {
        let expr = parse(expr);
        let t = typecheck(&expr);
        assert!(t.is_err(),
                "This expression should not typecheck: {:?}",
                expr);
    }

    #[test]
    fn test_arithmetics() {
        assert_valid("92", Int);
        assert_valid("true", Bool);

        assert_valid("1 + 1", Int);
        assert_fails("1 * true");
    }

    #[test]
    fn test_bools() {
        assert_valid("1 < 1", Bool);
        assert_fails("true == true");
        assert_fails("false > 92");
    }

    #[test]
    fn test_if() {
        assert_valid("if 1 < 2 then 92 else 62", Int);
        assert_valid("if true then false else true", Bool);
        assert_fails("if 1 + (1 == 2) then 92 else 62");
        assert_fails("if 1 then 92 else 62");
        assert_fails("if true then 92 else false");
    }

    #[test]
    fn test_fun() {
        assert_valid("fun id (x: int): int is x", Int.maps_to(Int));
        assert_valid("fun id (x: int): int is id x", Int.maps_to(Int));
        assert_valid("(fun id (x: int): int is x) 92", Int);

        assert_fails("fun id (x: int): int is y");
        assert_fails("(fun id (x: int): int is x) true");
    }

    #[test]
    fn test_let_fun() {
        assert_valid("let fun inc (x: int): int is x + 1 in inc 92", Int);

        assert_fails("let fun inc (x: int): int is x + 1 in inc inc");
    }
}
