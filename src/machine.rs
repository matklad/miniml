use std::fmt;

struct Name(String);

struct Frame {
    instructions: Vec<Instruction>
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Bool(bool),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Int(i) => i.fmt(f),
            Value::Bool(b) => b.fmt(f),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Instruction {
    ArithInstruction(ArithInstruction),
    CmpInstruction(CmpInstruction),
    PushInt(i64),
    PushBool(bool),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Instruction::*;
        match *self {
            ArithInstruction(ref inst) => inst.fmt(f),
            CmpInstruction(ref inst) => inst.fmt(f),
            PushInt(i) => write!(f, "push {}", i),
            PushBool(b) => write!(f, "push {}", b),
        }
    }
}

#[derive(Clone, Copy)]
pub enum ArithInstruction {
    Add,
    Sub,
    Mul,
    Div
}

impl fmt::Display for ArithInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ArithInstruction::*;
        f.write_str(match *self {
            Add => "add",
            Sub => "sub",
            Mul => "mul",
            Div => "div"
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

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Instruction as fmt::Display>::fmt(self, f)
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
}

fn runtime_error(message: &str) -> RuntimeError {
    RuntimeError {
        message: message.to_owned()
    }
}

pub type Result<T> = ::std::result::Result<T, RuntimeError>;

#[derive(Debug)]
pub struct Machine {
    instructions: Vec<Instruction>,
    state: State,
}

impl Machine {
    pub fn new(instructions: Vec<Instruction>) -> Machine {
        Machine {
            instructions: instructions,
            state: State::initial(),
        }
    }

    pub fn exec(&mut self) -> Result<Value> {
        while let Some(inst) = self.current_instruction() {
            try!(inst.exec(&mut self.state));
        }
        self.state.pop_value()
    }

    fn current_instruction(&self) -> Option<Instruction> {
        let ip = self.state.ip;
        assert!(ip <= self.instructions.len());
        self.instructions.get(ip).cloned()
    }

}

#[derive(Debug)]
struct State {
    ip: usize,
    values: Vec<Value>,
}

impl State {
    fn initial() -> State {
        State {
            ip: 0,
            values: Vec::new(),
        }
    }

    fn push_int(&mut self, value: i64) {
        self.values.push(Value::Int(value))
    }

    fn push_bool(&mut self, value: bool) {
        self.values.push(Value::Bool(value))
    }

    fn pop_int(&mut self) -> Result<i64> {
        self.pop_value()
            .and_then(|value| match value {
                Value::Int(i) => Ok(i),
                _ => Err(runtime_error("runtime type error :("))
            })
    }

    fn pop_value(&mut self) -> Result<Value> {
        self.values.pop()
            .ok_or(runtime_error("empty stack!"))
    }
}

trait Exec {
    fn exec(self, state: &mut State) -> Result<()>;
}

impl Exec for Instruction {
    fn exec(self, state: &mut State) -> Result<()> {
        use self::Instruction::*;

        match self {
            ArithInstruction(inst) => try!(inst.exec(state)),
            CmpInstruction(inst) => try!(inst.exec(state)),
            PushInt(i) => state.push_int(i),
            PushBool(b) => state.push_bool(b),
        }
        state.ip += 1;
        Ok(())
    }
}

impl Exec for ArithInstruction {
    fn exec(self, state: &mut State) -> Result<()> {
        let op1 = try!(state.pop_int());
        let op2 = try!(state.pop_int());
        state.push_int(op1 + op2);
        Ok(())
    }
}

impl Exec for CmpInstruction {
    fn exec(self, state: &mut State) -> Result<()> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const add: Instruction = Instruction::ArithInstruction(ArithInstruction::Add);

    fn pushi(i: i64) -> Instruction {
        Instruction::PushInt(i)
    }

    fn assert_execs(instructions: Vec<Instruction>, expected: Value) {
        let mut machine = Machine::new(instructions);
        match machine.exec() {
            Ok(value) => assert_eq!(value, expected),
            Err(e) => assert!(false, "Machine panicked with error {:?}\n{:#?}", e, machine),
        }
    }

    #[test]
    fn simple_eval() {
        assert_execs(vec![pushi(90), pushi(2), add], Value::Int(92));
    }

}

