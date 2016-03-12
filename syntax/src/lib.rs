use std::fmt;

mod parser;
mod parser_util;

pub use self::exprs::{Expr, Literal, BinOp, ArithBinOp, ArithOp, CmpBinOp, CmpOp, If, Fun, LetFun, LetRec, Apply};
pub use self::types::Type;
pub use self::parser::parse_Expr as parse;
pub use self::parser::parse_Type as parse_type;

mod exprs;
mod types;

#[derive(PartialEq, Eq, Hash)]
pub struct Ident(String);

impl Ident {
    fn from_str(name: &str) -> Ident {
        Ident(name.to_owned())
    }
}

impl AsRef<str> for Ident {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
