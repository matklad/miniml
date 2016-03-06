extern crate syntax;

pub use syntax::parse;
pub use typecheck::typecheck;
pub use compile::compile;
pub use machine::Machine;

mod typecheck;
mod context;
mod compile;
mod machine;

#[cfg(test)]
mod tests;
