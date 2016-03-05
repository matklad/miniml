use std::fmt;

struct Name(String);

type Frame = Vec<Instruction>;

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
//    Branch(usize, usize),
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
    RuntimeError { message: message.to_owned() }
}

fn fatal_error(message: &str) -> RuntimeError {
    RuntimeError { message: format!("Fatal: {} :(", message) }
}

pub type Result<T> = ::std::result::Result<T, RuntimeError>;

#[derive(Debug)]
pub struct Machine {
    frames: Vec<Frame>,
    state: State,
}

impl Machine {
    pub fn new(instructions: Vec<Instruction>) -> Machine {
        Machine {
            frames: vec![instructions],
            state: State::initial(),
        }
    }

    pub fn exec(&mut self) -> Result<Value> {
        while let Some(inst) = self.current_instruction() {
            try!(inst.exec(&mut self.state));
        }
        let result = try!(self.state.pop_value());
        if !self.state.values.is_empty() {
            return Err(fatal_error("more then one value on stack left"));
        }
        Ok(result)
    }

    fn current_instruction(&self) -> Option<Instruction> {
        let ip = self.state.ip;
        let frame = self.current_frame();
        assert!(ip <= frame.len());
        frame.get(ip).cloned()
    }

    fn current_frame(&self) -> &[Instruction] {
        &self.frames[self.state.fp]
    }
}

#[derive(Debug)]
struct State {
    ip: usize,
    fp: usize,
    values: Vec<Value>,
}

impl State {
    fn initial() -> State {
        State {
            ip: 0,
            fp: 0,
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
            .and_then(|value| {
                match value {
                    Value::Int(i) => Ok(i),
                    _ => Err(fatal_error("runtime type error")),
                }
            })
    }

    fn pop_value(&mut self) -> Result<Value> {
        self.values
            .pop()
            .ok_or(fatal_error("empty stack"))
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
        use self::ArithInstruction::*;
        let op2 = try!(state.pop_int());
        let op1 = try!(state.pop_int());
        let ret = match self {
            Add => op1 + op2,
            Sub => op1 - op2,
            Mul => op1 * op2,
            Div => {
                if op2 == 0 {
                    return Err(runtime_error("Division by zero"));
                } else {
                    op1 / op2
                }
            }
        };
        state.push_int(ret);
        Ok(())
    }
}

impl Exec for CmpInstruction {
    fn exec(self, state: &mut State) -> Result<()> {
        use self::CmpInstruction::*;
        let op2 = try!(state.pop_int());
        let op1 = try!(state.pop_int());
        let ret = match self {
            Lt => op1 < op2,
            Eq => op1 == op2,
            Gt => op1 > op2,
        };
        state.push_bool(ret);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADD: Instruction = Instruction::ArithInstruction(ArithInstruction::Add);
    const SUB: Instruction = Instruction::ArithInstruction(ArithInstruction::Sub);
    const MUL: Instruction = Instruction::ArithInstruction(ArithInstruction::Mul);
    const DIV: Instruction = Instruction::ArithInstruction(ArithInstruction::Div);

    const LT: Instruction = Instruction::CmpInstruction(CmpInstruction::Lt);
    const EQ: Instruction = Instruction::CmpInstruction(CmpInstruction::Eq);
    const GT: Instruction = Instruction::CmpInstruction(CmpInstruction::Gt);

    fn pushi(i: i64) -> Instruction {
        Instruction::PushInt(i)
    }

    fn pushb(b: bool) -> Instruction {
        Instruction::PushBool(b)
    }

    fn assert_execs(instructions: Vec<Instruction>, expected: Value) {
        let mut machine = Machine::new(instructions);
        match machine.exec() {
            Ok(value) => {
                assert!(value == expected,
                        "Wrong answer\nExpected {:?}\nGot {:?}\nMachine {:#?}",
                        expected,
                        value,
                        machine)
            }
            Err(e) => assert!(false, "Machine panicked with error {:?}\n{:#?}", e, machine),
        }
    }

    fn assert_fails(instructions: Vec<Instruction>, expected_message: &str) {
        let mut machine = Machine::new(instructions);
        match machine.exec() {
            Ok(_) => {
                assert!(false,
                        "Machine should have failed with {}\n{:#?}",
                        expected_message,
                        machine)
            }
            Err(e) => {
                assert!(e.message.contains(expected_message),
                        "Wrong error message.\nExpected: {}\nGot:      {}\n{:#?}",
                        expected_message,
                        e.message,
                        machine)
            }
        }
    }

    #[test]
    fn basic() {
        assert_execs(vec![pushi(92)], Value::Int(92));
        assert_fails(vec![], "Fatal: empty stack :(");
        assert_fails(vec![pushi(92), pushi(62)],
                     "Fatal: more then one value on stack left :(")
    }

    #[test]
    fn arith() {
        assert_execs(vec![pushi(90), pushi(2), ADD], Value::Int(92));
        assert_execs(vec![pushi(94), pushi(2), SUB], Value::Int(92));
        assert_execs(vec![pushi(46), pushi(2), MUL], Value::Int(92));
        assert_execs(vec![pushi(184), pushi(2), DIV], Value::Int(92));

        assert_fails(vec![pushi(1), pushi(0), DIV], "Division by zero");
        assert_fails(vec![ADD], "Fatal: empty stack :(");
        assert_fails(vec![pushi(1), pushb(true), ADD],
                     "Fatal: runtime type error :(");
    }

    #[test]
    fn cmp() {
        assert_execs(vec![pushi(92), pushi(62), LT], Value::Bool(false));
        assert_execs(vec![pushi(92), pushi(62), GT], Value::Bool(true));
        assert_execs(vec![pushi(1), pushi(2), EQ], Value::Bool(false));
        assert_execs(vec![pushi(2), pushi(2), EQ], Value::Bool(true));

        assert_fails(vec![pushi(1), pushb(true), EQ], "Fatal: runtime type error :(");
        assert_fails(vec![pushb(false), pushb(true), EQ], "Fatal: runtime type error :(");
    }

}
