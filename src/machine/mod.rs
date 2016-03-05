use std::fmt;
pub use self::value::Value;

mod value;

struct Name(String);

type Frame = Vec<Instruction>;

#[derive(Clone, Copy)]
pub enum Instruction {
    ArithInstruction(ArithInstruction),
    CmpInstruction(CmpInstruction),
    PushInt(i64),
    PushBool(bool),
    Branch(usize, usize),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Instruction::*;
        match *self {
            ArithInstruction(ref inst) => inst.fmt(f),
            CmpInstruction(ref inst) => inst.fmt(f),
            PushInt(i) => write!(f, "push {}", i),
            PushBool(b) => write!(f, "push {}", b),
            Branch(l, r) => write!(f, "branch {} {}", l, r),
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
            try!(inst.exec(self));
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

    fn switch_frame(&mut self, frame: usize) -> Result<()> {
        if frame > self.frames.len() {
            return Err(fatal_error("illegal jump"));
        }
        self.state.fp = frame;
        self.state.ip = 0;
        Ok(())
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
        self.pop_value().and_then(|v| v.into_int())
    }

    fn pop_bool(&mut self) -> Result<bool> {
        self.pop_value().and_then(|v| v.into_bool())
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

impl Instruction {
    fn exec(self, machine: &mut Machine) -> Result<()> {
        use self::Instruction::*;

        match self {
            ArithInstruction(inst) => try!(inst.exec(&mut machine.state)),
            CmpInstruction(inst) => try!(inst.exec(&mut machine.state)),
            PushInt(i) => machine.state.push_int(i),
            PushBool(b) => machine.state.push_bool(b),
            Branch(tru, fls) => {
                let cond = try!(machine.state.pop_bool());
                return machine.switch_frame(if cond {
                    tru
                } else {
                    fls
                });
            }
        }
        machine.state.ip += 1;
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
    use std::str::FromStr;
    use super::*;

    fn parse_secd(input: &str) -> Vec<Instruction> {
        fn parse_inst(line: &str) -> Instruction {
            let mut words = line.trim().split_whitespace();
            let op = words.next().expect("Missing op");
            let arg = words.next().map(|s| {
                match s {
                    "true" => Value::Bool(true),
                    "false" => Value::Bool(false),
                    _ => Value::Int(i64::from_str(s).unwrap()),
                }
            });
            assert_eq!(None, words.next());
            match (op, arg) {
                ("add", None) => Instruction::ArithInstruction(ArithInstruction::Add),
                ("sub", None) => Instruction::ArithInstruction(ArithInstruction::Sub),
                ("mul", None) => Instruction::ArithInstruction(ArithInstruction::Mul),
                ("div", None) => Instruction::ArithInstruction(ArithInstruction::Div),
                ("lt", None) => Instruction::CmpInstruction(CmpInstruction::Lt),
                ("eq", None) => Instruction::CmpInstruction(CmpInstruction::Eq),
                ("gt", None) => Instruction::CmpInstruction(CmpInstruction::Gt),
                ("push", Some(Value::Int(i))) => Instruction::PushInt(i),
                ("push", Some(Value::Bool(b))) => Instruction::PushBool(b),
                _ => panic!("Invalid instruction {}", line),
            }

        }
        let input = input.trim();
        input.lines()
             .flat_map(|line| line.split(';'))
             .map(parse_inst)
             .collect()
    }

    fn assert_execs<V: Into<Value>>(expected: V, asm: &str) {
        let expected = expected.into();
        let mut machine = Machine::new(parse_secd(asm));
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

    fn assert_fails(expected_message: &str, asm: &str) {
        let mut machine = Machine::new(parse_secd(asm));
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
        assert_execs(92, "push 92");
        assert_fails("Fatal: empty stack :(", "");
        assert_fails("Fatal: more then one value on stack left :(",
                     "push 1; push 2");
    }

    #[test]
    fn arith() {
        assert_execs(92, "push 90; push 2; add");
        assert_execs(92, "push 94; push 2; sub");
        assert_execs(92, "push 46; push 2; mul ");
        assert_execs(92, "push 184; push 2; div");

        assert_fails("Division by zero", "push 1; push 0; div");
        assert_fails("Fatal: empty stack :(", "add");
        assert_fails("Fatal: runtime type error :(", "push 1; push true; add");
    }

    #[test]
    fn cmp() {
        assert_execs(false, "push 92; push 62; lt");
        assert_execs(true, "push 92; push 62; gt");
        assert_execs(false, "push 1; push 2; eq");
        assert_execs(true, "push 2; push 2; eq");

        assert_fails("Fatal: runtime type error :(", "push 1; push true; eq");
        assert_fails("Fatal: runtime type error :(", "push true; push false; eq");
    }
}
