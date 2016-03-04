use std::fmt;

mod parser;
mod parser_util;

pub use self::exprs::{Expr, Literal, ArithBinOp, CmpBinOp, If, Fun};
pub use self::types::Type;
pub use self::parser::parse_Expr as parse;

mod exprs;
mod types;

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
