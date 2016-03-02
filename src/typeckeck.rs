use syntax::{Expr, Literal, ArithBinOp, CmpBinOp, If, Type};

pub type Result = ::std::result::Result<Type, TypeError>;

#[derive(Debug)]
pub struct TypeError {
    pub message: String,
}

pub fn typecheck(expr: &Expr) -> Result {
    expr.check()
}

macro_rules! bail {
    ($msg:expr) => { bail!($e, $msg,) };

    ($msg:expr, $($farg:expr),*) => {
        return Err(TypeError {
            message: format!($msg $(, $farg)*),
        })
    };
}

fn expect(expr: &Expr, type_: Type) -> Result {
    let t = try!(expr.check());
    if t != type_ {
        bail!("Expected {:?}, got {:?}", type_, t);
    }
    Ok(type_)
}

trait Typecheck {
    fn check(&self) -> Result;
}

impl Typecheck for Expr {
    fn check(&self) -> Result {
        use syntax::Expr::*;
        match *self {
            Literal(ref l) => l.check(),
            ArithBinOp(ref op) => op.check(),
            CmpBinOp(ref op) => op.check(),
            If(ref if_) => if_.check(),
            _ => unimplemented!(),
        }
    }
}

impl Typecheck for Literal {
    fn check(&self) -> Result {
        let t = match *self {
            Literal::Number(_) => Type::Int,
            Literal::Bool(_) => Type::Bool,
        };
        Ok(t)
    }
}

impl Typecheck for ArithBinOp {
    fn check(&self) -> Result {
        try!(expect(&self.lhs, Type::Int));
        try!(expect(&self.rhs, Type::Int));
        Ok(Type::Int)
    }
}

impl Typecheck for CmpBinOp {
    fn check(&self) -> Result {
        try!(expect(&self.lhs, Type::Int));
        try!(expect(&self.rhs, Type::Int));
        Ok(Type::Bool)
    }
}

impl Typecheck for If {
    fn check(&self) -> Result {
        try!(expect(&self.cond, Type::Bool));
        let t1 = try!(self.tru.check());
        let t2 = try!(self.fls.check());
        if t1 != t2 {
            bail!("Arms of an if have different types: {:?} {:?}", t1, t2);
        }
        Ok(t1)
    }
}

#[cfg(test)]
mod tests {
    use syntax::{Expr, Type};
    use super::*;

    fn parse(expr: &str) -> Expr {
        ::syntax::parse(expr).expect(&format!("Failed to parse {}", expr))
    }

    fn assert_valid(expr: &str, type_: Type) {
        let expr = parse(expr);
        match typecheck(&expr) {
            Ok(t) => assert!(t == type_, "Wrong type for {:?}.\nExpected {:?}, got {:?}",
                             expr, type_, t),
            Err(e) => assert!(false, "Failed to typecheck {:?}:\n {:?}", expr, e),
        }
    }

    fn assert_fails(expr: &str) {
        let expr = parse(expr);
        let t = typecheck(&expr);
        assert!(t.is_err(), "This expression should not typecheck: {:?}", expr);
    }

    #[test]
    fn test_arithmetics() {
        assert_valid("92", Type::Int);
        assert_valid("true", Type::Bool);

        assert_valid("1 + 1", Type::Int);
        assert_fails("1 * true");
    }

    #[test]
    fn test_bools() {
        assert_valid("1 < 1", Type::Bool);
        assert_fails("true == true");
        assert_fails("false > 92");
    }

    #[test]
    fn test_if() {
        assert_valid("if 1 < 2 then 92 else 62", Type::Int);
        assert_valid("if true then false else true", Type::Bool);
        assert_fails("if 1 + (1 == 2) then 92 else 62");
        assert_fails("if 1 then 92 else 62");
        assert_fails("if true then 92 else false");
    }
}
