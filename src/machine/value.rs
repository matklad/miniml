use std::fmt;

use machine::{Result, fatal_error};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Bool(bool),
}

impl Value {
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
}

impl From<i64> for Value {
    fn from(i: i64) -> Value {
        Value::Int(i)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Value {
        Value::Bool(b)
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Int(i) => i.fmt(f),
            Value::Bool(b) => b.fmt(f),
        }
    }
}
