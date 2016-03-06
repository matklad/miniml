use std::fmt;

use machine::{Result, fatal_error};
use machine::program::{Name, Frame};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Value<'p> {
    Int(i64),
    Bool(bool),
    Closure(Closure<'p>),
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Closure<'p> {
    pub arg: Name,
    pub frame: &'p Frame,
    pub env: usize,
}

impl<'p> Value<'p> {
    pub fn into_int(self) -> Result<i64> {
        match self {
            Value::Int(i) => Ok(i),
            _ => Err(fatal_error("runtime type error")),
        }
    }

    pub fn into_bool(self) -> Result<bool> {
        match self {
            Value::Bool(b) => Ok(b),
            _ => Err(fatal_error("runtime type error")),
        }
    }

    pub fn into_closure(self) -> Result<Closure<'p>> {
        match self {
            Value::Closure(c) => Ok(c),
            _ => Err(fatal_error("runtime type error")),
        }
    }
}

impl From<i64> for Value<'static> {
    fn from(i: i64) -> Self {
        Value::Int(i)
    }
}

impl From<bool> for Value<'static> {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl<'p> fmt::Display for Value<'p> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Int(i) => i.fmt(f),
            Value::Bool(b) => b.fmt(f),
            Value::Closure(_) => "<closure>".fmt(f),
        }
    }
}

impl<'p> fmt::Debug for Value<'p> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Value as fmt::Display>::fmt(self, f)
    }
}
