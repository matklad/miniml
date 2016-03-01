mod parser;
mod parser_util;

pub use self::exprs::{Expr, ArithBinOp, ArithOp, CmpBinOp, CmpOp, Literal};
pub use self::parser::parse_Expr as parse;

//pub use self::types::Type;
//type SymbolTable = Vec<String>;
//
//pub struct Ident(IdentInner);
//
//enum IdentInner {
//    Interned(usize),
//    Owned(String),
//}
//
//mod types {
//    pub enum Type {
//        Int,
//        Bool,
//        Arrow(Arrow),
//    }
//
//    pub struct Arrow(Box<Type>, Box<Type>);
//}

mod exprs {
//    use super::{Type, Ident};
    use std::fmt::{self, Write};

    pub type Ident = String;

    pub enum Expr {
        Var(Ident),
        Literal(Literal),
        ArithBinOp(ArithBinOp),
        CmpBinOp(CmpBinOp),
        If(If),
//        Fun(Fun),
//        Application(Application),
    }

    impl fmt::Debug for Expr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            use self::Expr::*;
            match *self {
                Var(ref s) => f.write_str(s),
                Literal(ref l) => l.fmt(f),
                ArithBinOp(ref op) => op.fmt(f),
                CmpBinOp(ref op) => op.fmt(f),
                If(ref if_) => if_.fmt(f),
            }
        }
    }

    pub struct BinOp<T> {
        pub kind: T,
        pub lhs: Box<Expr>,
        pub rhs: Box<Expr>,
    }

    impl<T: fmt::Debug> fmt::Debug for BinOp<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
           write!(f, "({:?} {:?} {:?})", self.kind, self.lhs, self.rhs)
        }
    }

    pub enum ArithOp { Mul, Div, Add, Sub }

    impl fmt::Debug for ArithOp {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            use self::ArithOp::*;
            f.write_char(match *self {
                Mul => '*', Div => '\\',
                Add => '+', Sub => '-',
             })
        }
    }

    pub type ArithBinOp = BinOp<ArithOp>;

    pub enum CmpOp { Eq, Lt, Gt }

    impl fmt::Debug for CmpOp {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            use self::CmpOp::*;
            f.write_str(match *self {
                Eq => "==",
                Lt => "<",
                Gt => ">",
             })
        }
    }

    pub type CmpBinOp = BinOp<CmpOp>;

    pub struct If {
        pub cond: Box<Expr>,
        pub tru: Box<Expr>,
        pub fls: Box<Expr>,
    }

    impl fmt::Debug for If {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "(if {:?} {:?} {:?})", self.cond, self.tru, self.fls)
        }
    }

    pub enum Literal {
        Number(i64),
        Bool(bool),
    }

    impl fmt::Debug for Literal {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                Literal::Number(x) => x.fmt(f),
                Literal::Bool(b) => b.fmt(f),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parser;

    fn assert_parses(expr: &str, ast: &str) {
        let result = parser::parse_Expr(expr);
        assert!(result.is_ok(), "\n`{}` failed to parse:\n {:?}\n", expr, result);
        let result = format!("{:?}", result.unwrap());
        assert_eq!(result, ast);
    }

    fn you_shall_not_parse(expr: &str) {
        assert!(parser::parse_Expr(expr).is_err());
    }

    #[test]
    fn test_good() {
        assert_parses("92", "92");
        assert_parses("(92)", "92");
        assert_parses("(((92)))", "92");
        assert_parses("true", "true");
        assert_parses("false", "false");
        assert_parses("spam", "spam");
        assert_parses("1 == 1", "(== 1 1)");
        assert_parses("1 < 1 + 1", "(< 1 (+ 1 1))");
        assert_parses("1 * 2 > 1", "(> (* 1 2) 1)");
        assert_parses("(1 == 2) == 3", "(== (== 1 2) 3)");
        assert_parses("1 < (2 > 3)", "(< 1 (> 2 3))");
        assert_parses("1 + 2 * 3", "(+ 1 (* 2 3))");
        assert_parses("if 1 then 2 else if 3 then 4 else 5", "(if 1 2 (if 3 4 5))");
        assert_parses("if 1 then if 2 then 3 else 4 else 5", "(if 1 (if 2 3 4) 5)");
    }

    #[test]
    fn test_bad() {
        you_shall_not_parse("((92)");
        you_shall_not_parse("1 == 1 == 1");
        you_shall_not_parse("1 < 1 > 1");
    }
}
