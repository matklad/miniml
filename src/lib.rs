extern crate syntax;

pub use syntax::parse;
pub use compile::compile;
pub use typecheck::typecheck;
pub use machine::Machine;

mod typecheck;
mod ir;
mod context;
mod compile;
mod machine;

#[cfg(test)]
mod tests;
