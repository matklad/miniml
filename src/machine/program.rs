use std::fmt;

#[derive(Debug)]
pub struct Program {
    pub frames: Vec<Frame>,
}

pub type Frame = Vec<Instruction>;

#[derive(Clone, Copy)]
pub enum Instruction {
    ArithInstruction(ArithInstruction),
    CmpInstruction(CmpInstruction),
    PushInt(i64),
    PushBool(bool),
    Branch(usize, usize),
    Var(Name),
    Closure {
        name: Name,
        arg: Name,
        frame: usize,
    },
    Call,
    PopEnv,
}

pub type Name = usize;

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Instruction::*;
        match *self {
            ArithInstruction(ref inst) => inst.fmt(f),
            CmpInstruction(ref inst) => inst.fmt(f),
            PushInt(i) => write!(f, "push {}", i),
            PushBool(b) => write!(f, "push {}", b),
            Branch(l, r) => write!(f, "branch {} {}", l, r),
            Var(n) => write!(f, "var {}", n),
            Closure { name, arg, frame} => write!(f, "clos {} {} {}", name, arg, frame),
            Call => "call".fmt(f),
            PopEnv => "ret".fmt(f),
        }
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Instruction as fmt::Display>::fmt(self, f)
    }
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
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
