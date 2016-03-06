extern crate syntax;

pub use syntax::parse;
pub use typecheck::typecheck;

mod typecheck;
mod context;
mod compile;
mod machine;

#[cfg(test)]
mod tests;
