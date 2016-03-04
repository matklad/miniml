use syntax::{Expr, Literal, ArithBinOp, CmpBinOp, If, Fun, Apply, Type};
use context::{Context, StackContext};

pub type Result = ::std::result::Result<Type, TypeError>;

#[derive(Debug)]
pub struct TypeError {
    pub message: String,
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

fn expect<'a, 'c: 'a, C: Context<'c, Type>>(expr: &'c Expr, type_: Type, ctx: &'a mut C) -> Result {
    let t = try!(expr.check(ctx));
    if t != type_ {
        bail!("Expected {:?}, got {:?}", type_, t);
    }
    Ok(type_)
}

trait Typecheck {
    fn check<'a, 'c: 'a, C: Context<'c, Type>>(&'c self, ctx: &'a mut C) -> Result;
}

impl Typecheck for Expr {
    fn check<'a, 'c: 'a, C: Context<'c, Type>>(&'c self, ctx: &'a mut C) -> Result {
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
            Apply(ref apply) => apply.check(ctx),
        }
    }
}

impl Typecheck for Literal {
    fn check<'a, 'c: 'a, C: Context<'c, Type>>(&'c self, _: &'a mut C) -> Result {
        let t = match *self {
            Literal::Number(_) => Type::Int,
            Literal::Bool(_) => Type::Bool,
        };
        Ok(t)
    }
}

impl Typecheck for ArithBinOp {
    fn check<'a, 'c: 'a, C: Context<'c, Type>>(&'c self, ctx: &'a mut C) -> Result {
        try!(expect(&self.lhs, Type::Int, ctx));
        try!(expect(&self.rhs, Type::Int, ctx));
        Ok(Type::Int)
    }
}

impl Typecheck for CmpBinOp {
    fn check<'a, 'c: 'a, C: Context<'c, Type>>(&'c self, ctx: &'a mut C) -> Result {
        try!(expect(&self.lhs, Type::Int, ctx));
        try!(expect(&self.rhs, Type::Int, ctx));
        Ok(Type::Bool)
    }
}

impl Typecheck for If {
    fn check<'a, 'c: 'a, C: Context<'c, Type>>(&'c self, ctx: &'a mut C) -> Result {
        try!(expect(&self.cond, Type::Bool, ctx));
        let t1 = try!(self.tru.check(ctx));
        let t2 = try!(self.fls.check(ctx));
        if t1 != t2 {
            bail!("Arms of an if have different types: {:?} {:?}", t1, t2);
        }
        Ok(t1)
    }
}

impl Typecheck for Fun {
    fn check<'a, 'c: 'a, C: Context<'c, Type>>(&'c self, ctx: &'a mut C) -> Result {
        let result = Type::arrow(&self.arg_type, &self.fun_type);
        ctx.push(&self.arg_name, self.arg_type.clone());
        ctx.push(&self.name, result.clone());
        try!(expect(&self.body, self.fun_type.clone(), ctx));
        ctx.pop();
        ctx.pop();
        Ok(result)
    }
}

impl Typecheck for Apply {
    fn check<'a, 'c: 'a, C: Context<'c, Type>>(&'c self, ctx: &'a mut C) -> Result {
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
    use syntax::{Expr, Type};
    use super::*;

    fn parse(expr: &str) -> Expr {
        ::syntax::parse(expr).expect(&format!("Failed to parse {}", expr))
    }

    fn parse_type(type_: &str) -> Type {
        ::syntax::parse_type(type_).expect(&format!("Failed to parse {}", type_))
    }

    fn assert_valid(expr: &str, type_: &str) {
        let expr = parse(expr);
        let type_ = parse_type(type_);
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
        assert_valid("92", "int");
        assert_valid("true", "bool");

        assert_valid("1 + 1", "int");
        assert_fails("1 * true");
    }

    #[test]
    fn test_bools() {
        assert_valid("1 < 1", "bool");
        assert_fails("true == true");
        assert_fails("false > 92");
    }

    #[test]
    fn test_if() {
        assert_valid("if 1 < 2 then 92 else 62", "int");
        assert_valid("if true then false else true", "bool");
        assert_fails("if 1 + (1 == 2) then 92 else 62");
        assert_fails("if 1 then 92 else 62");
        assert_fails("if true then 92 else false");
    }

    #[test]
    fn test_fun() {
        assert_valid("fun id (x: int): int is x", "int -> int");
        assert_valid("fun id (x: int): int is id x", "int -> int");
    }
}
