use std::fmt;

pub type Frame = Vec<Instruction>;

#[derive(PartialEq, Eq, Debug)]
pub enum Instruction {
    ArithInstruction(ArithInstruction),
    CmpInstruction(CmpInstruction),
    PushInt(i64),
    PushBool(bool),
    Branch(Frame, Frame),
    Var(Name),
    Closure {
        name: Name,
        arg: Name,
        frame: Frame,
    },
    Call,
    PopEnv,
}

pub type Name = usize;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ArithInstruction {
    Add,
    Sub,
    Mul,
    Div,
}

impl fmt::Display for ArithInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ArithInstruction::*;
        f.write_str(match *self {
            Add => "add",
            Sub => "sub",
            Mul => "mul",
            Div => "div",
        })
    }
}

impl fmt::Debug for ArithInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <ArithInstruction as fmt::Display>::fmt(self, f)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CmpInstruction {
    Lt,
    Eq,
    Gt,
}

impl fmt::Display for CmpInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::CmpInstruction::*;
        f.write_str(match *self {
            Lt => "lt",
            Eq => "eq",
            Gt => "gt",
        })
    }
}

impl fmt::Debug for CmpInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <CmpInstruction as fmt::Display>::fmt(self, f)
    }
}
