use std::fmt;
pub use self::value::Value;

mod value;

//struct Name(String);

pub type Frame = Vec<Instruction>;

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
    pub fn new(frames: Vec<Frame>) -> Machine {
        Machine {
            frames: if frames.is_empty() {
                vec![vec![]]
            } else {
                frames
            },
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
        let ip = self.state.ip();
        let frame = self.current_frame();
        assert!(ip <= frame.len());
        frame.get(ip).cloned()
    }

    fn current_frame(&self) -> &[Instruction] {
        assert!(self.state.fp() < self.frames.len(),
                "no such frame {}",
                self.state.fp());
        &self.frames[self.state.fp()]
    }

    fn switch_frame(&mut self, frame: usize) -> Result<()> {
        if frame > self.frames.len() {
            return Err(fatal_error("illegal jump"));
        }
        self.state.activations.push(Activation {
            ip: 0,
            fp: frame,
        });
        Ok(())
    }
}

#[derive(Debug)]
struct State {
    values: Vec<Value>,
    activations: Vec<Activation>,
}

#[derive(Debug)]
struct Activation {
    ip: usize,
    fp: usize,
}

impl State {
    fn initial() -> State {
        State {
            values: Vec::new(),
            activations: vec![Activation {
                ip: 0,
                fp: 0,
            }],
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

    fn fp(&self) -> usize {
        self.activation().fp
    }

    fn ip(&self) -> usize {
        self.activation().ip
    }

    fn inc_ip(&mut self) {
        self.activations.last_mut().unwrap().ip += 1;
    }

    fn activation(&self) -> &Activation {
        self.activations.last().unwrap()
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
        machine.state.inc_ip();
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
    use itertools::Itertools;

    use super::*;

    fn parse_secd(input: &str) -> Vec<Frame> {
        fn parse_inst(line: &str) -> Instruction {
            let mut words = line.split_whitespace();
            let op = words.next().expect("Missing op");
            let result = {
                let mut much_arg = || words.next().expect("Missing arg");
                match op {
                    "add" => Instruction::ArithInstruction(ArithInstruction::Add),
                    "sub" => Instruction::ArithInstruction(ArithInstruction::Sub),
                    "mul" => Instruction::ArithInstruction(ArithInstruction::Mul),
                    "div" => Instruction::ArithInstruction(ArithInstruction::Div),
                    "lt" => Instruction::CmpInstruction(CmpInstruction::Lt),
                    "eq" => Instruction::CmpInstruction(CmpInstruction::Eq),
                    "gt" => Instruction::CmpInstruction(CmpInstruction::Gt),
                    "push" => {
                        match much_arg() {
                            "true" => Instruction::PushBool(true),
                            "false" => Instruction::PushBool(false),
                            s => Instruction::PushInt(i64::from_str(s).unwrap()),
                        }
                    }
                    "branch" => {
                        let mut much_usize_arg = || usize::from_str(much_arg()).unwrap();
                        let tru = much_usize_arg();
                        let fls = much_usize_arg();
                        Instruction::Branch(tru, fls)
                    }
                    _ => panic!("Unknown op: {}", op),
                }
            };
            assert_eq!(None, words.next());
            result
        }

        let input = input.trim();
        input.lines()
             .flat_map(|line| line.split(';'))
             .map(|line| line.trim())
             .group_by_lazy(|line| line.is_empty())
             .into_iter()
             .filter(|&(is_blank, _)| !is_blank)
             .map(|(_, frame)| frame.into_iter().map(parse_inst).collect::<Frame>())
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

    #[test]
    fn branch() {
        assert_execs(92,
                     "
            push true
            branch 1 2

            push 92

            push 62
        ");

        assert_execs(62,
                     "
            push false
            branch 1 2

            push 92

            push 62
        ");

        assert_fails("Fatal: runtime type error :(",
                     "
            push 92
            branch 1 2

            push true

            push false
        ");

        assert_execs(92,"
            push true
            branch 1 2
            push false
            branch 1 2
            add

            push 41

            push 51
        ");

        assert_fails("Fatal: illegal jump :(", "push true; branch 92 92");
    }
}
