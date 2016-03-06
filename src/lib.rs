extern crate syntax;
extern crate itertools;

pub use syntax::parse;
pub use typecheck::typecheck;

mod typecheck;
mod context;
mod machine;
