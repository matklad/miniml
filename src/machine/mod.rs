use self::program::{Program, Instruction, ArithInstruction, CmpInstruction};
pub use self::value::Value;

mod value;
mod program;

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

type Activation<'p> = &'p [Instruction];

#[derive(Debug)]
pub struct Machine<'p> {
    program: &'p Program,
    values: Vec<Value>,
    activations: Vec<Activation<'p>>,
}

impl<'p> Machine<'p> {
    pub fn new(program: &'p Program) -> Self {
        Machine {
            program: program,
            values: vec![],
            activations: vec![],
        }
    }

    pub fn exec(&mut self) -> Result<Value> {
        try!(self.switch_frame(0));

        while let Some(inst) = self.fetch_instruction() {
            try!(inst.exec(self));
        }

        self.pop_value().and_then(|result| {
            if !self.values.is_empty() {
                return Err(fatal_error("more then one value on stack left"));
            }
            Ok(result)
        })
    }

    fn fetch_instruction(&mut self) -> Option<Instruction> {
        self.activations.pop().and_then(|act| {
            act.split_first().map(|(inst, act)| {
                if !act.is_empty() {
                    self.activations.push(act);
                }
                *inst
            })
        })
    }

    fn switch_frame(&mut self, frame: usize) -> Result<()> {
        self.program
            .frames
            .get(frame)
            .ok_or(fatal_error("illegal jump"))
            .map(|frame| self.activations.push(frame))
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
    fn exec(self, state: &mut Machine) -> Result<()>;
}

impl Exec for Instruction {
    fn exec(self, machine: &mut Machine) -> Result<()> {
        use self::program::Instruction::*;

        match self {
            ArithInstruction(inst) => try!(inst.exec(machine)),
            CmpInstruction(inst) => try!(inst.exec(machine)),
            PushInt(i) => machine.push_int(i),
            PushBool(b) => machine.push_bool(b),
            Branch(tru, fls) => {
                let jump = if try!(machine.pop_bool()) {
                    tru
                } else {
                    fls
                };
                return machine.switch_frame(jump);
            }
        }
        Ok(())
    }
}

impl Exec for ArithInstruction {
    fn exec(self, machine: &mut Machine) -> Result<()> {
        use self::program::ArithInstruction::*;
        let op2 = try!(machine.pop_int());
        let op1 = try!(machine.pop_int());
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
        machine.push_int(ret);
        Ok(())
    }
}

impl Exec for CmpInstruction {
    fn exec(self, machine: &mut Machine) -> Result<()> {
        use self::program::CmpInstruction::*;
        let op2 = try!(machine.pop_int());
        let op1 = try!(machine.pop_int());
        let ret = match self {
            Lt => op1 < op2,
            Eq => op1 == op2,
            Gt => op1 > op2,
        };
        machine.push_bool(ret);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use itertools::Itertools;

    use super::*;
    use super::program::*;

    fn parse_secd(input: &str) -> Program {
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
        let frames = input.lines()
                          .flat_map(|line| line.split(';'))
                          .map(|line| line.trim())
                          .group_by_lazy(|line| line.is_empty())
                          .into_iter()
                          .filter(|&(is_blank, _)| !is_blank)
                          .map(|(_, frame)| frame.into_iter().map(parse_inst).collect::<Frame>())
                          .collect();
        Program { frames: frames }

    }

    fn assert_execs<V: Into<Value>>(expected: V, asm: &str) {
        let expected = expected.into();
        let program = parse_secd(asm);
        let mut machine = Machine::new(&program);
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
        let program = parse_secd(asm);
        let mut machine = Machine::new(&program);
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
        assert_fails("Fatal: illegal jump :(", "");
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

        assert_execs(92,
                     "
            push true
            branch 1 2
            push false
            branch 1 2
            add

            push 41

            push 51");

        assert_fails("Fatal: illegal jump :(", "push true; branch 92 92");
    }
}
