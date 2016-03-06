use std::collections::HashMap;
use self::program::{Program, Instruction, Name, ArithInstruction, CmpInstruction};
pub use self::value::{Value, Closure};

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
    storage: Vec<Env>,
    values: Vec<Value>,
    environments: Vec<Env>,
    activations: Vec<Activation<'p>>,
}

type Env = HashMap<Name, Value>;

impl<'p> Machine<'p> {
    pub fn new(program: &'p Program) -> Self {
        Machine {
            program: program,
            storage: vec![],
            values: vec![],
            environments: vec![Env::new()],
            activations: vec![],
        }
    }

    pub fn exec(&mut self) -> Result<Value> {
        try!(self.switch_frame(0));

        while let Some(inst) = self.fetch_instruction() {
            println!("\n{:?}", self.values);
            println!("{}", inst);
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
        self.push_value(Value::Int(value))
    }

    fn push_bool(&mut self, value: bool) {
        self.push_value(Value::Bool(value))
    }

    fn push_value(&mut self, value: Value) {
        self.values.push(value)
    }

    fn pop_int(&mut self) -> Result<i64> {
        self.pop_value().and_then(|v| v.into_int())
    }

    fn pop_bool(&mut self) -> Result<bool> {
        self.pop_value().and_then(|v| v.into_bool())
    }

    fn pop_closure(&mut self) -> Result<Closure> {
        self.pop_value().and_then(|v| v.into_closure())
    }

    fn pop_value(&mut self) -> Result<Value> {
        self.values
            .pop()
            .ok_or(fatal_error("empty stack"))
    }

    fn lookup(&mut self, name: Name) -> Result<Value> {
        self.current_env().get(&name).cloned().ok_or(fatal_error("undefined variable"))
    }

    fn current_env(&self) -> &Env {
        self.environments.last().unwrap()
    }

    fn pop_env(&mut self) -> Result<()> {
        if self.environments.len() == 0 {
            return Err(fatal_error("no environment"));
        }
        self.environments.pop();
        Ok(())
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
            Var(name) => {
                let value = try!(machine.lookup(name));
                machine.push_value(value);
            }
            Closure { name, arg, frame } => {
                let mut env = machine.current_env().clone();
                let env_idx = machine.storage.len();

                let value = Value::Closure(value::Closure {
                    arg: arg,
                    frame: frame,
                    env: env_idx,
                });
                env.insert(name, value);
                machine.storage.push(env);
                machine.push_value(value);
            }
            Call => {
                let arg_value = try!(machine.pop_value());
                let value::Closure { arg, frame, env } = try!(machine.pop_closure());
                let mut env = machine.storage[env].clone();
                env.insert(arg, arg_value);
                machine.environments.push(env);
                return machine.switch_frame(frame);
            }
            PopEnv => try!(machine.pop_env()),
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
                    "call" => Instruction::Call,
                    "ret" => Instruction::PopEnv,
                    "push" => {
                        match much_arg() {
                            "true" => Instruction::PushBool(true),
                            "false" => Instruction::PushBool(false),
                            s => Instruction::PushInt(i64::from_str(s).unwrap()),
                        }
                    }
                    "var" => Instruction::Var(Name::from_str(much_arg()).unwrap()),
                    "branch" => {
                        let mut much_usize_arg = || usize::from_str(much_arg()).unwrap();
                        let tru = much_usize_arg();
                        let fls = much_usize_arg();
                        Instruction::Branch(tru, fls)
                    }
                    "clos" => {
                        let mut munch_usize_arg = || usize::from_str(much_arg()).unwrap();
                        let name = munch_usize_arg();
                        let arg = munch_usize_arg();
                        let frame = munch_usize_arg();
                        Instruction::Closure {
                            name: name,
                            arg: arg,
                            frame: frame,
                        }
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

    #[test]
    fn vars() {
        assert_execs(92,
                     "
            clos 0 1 1
            push 92
            call

            var 1
           ");
        assert_fails("Fatal: undefined variable :(", "var 92");
    }

    #[test]
    fn factorial() {
        assert_execs(120,
                     "
            clos 0 1 1
            push 5
            call

            push 0
            var 1
            eq
            branch 2 3
            ret

            push 1

            var 1
            var 0
            var 1
            push 1
            sub
            call
            mul
           ");
    }

    #[test]
    fn hof() {
        assert_execs(92,
                     "
            clos 0 1 2
            clos 0 1 1
            call
            push 23
            call

            var 1
            var 1
            add
            ret

            clos 2 3 3
            ret

            var 1
            var 1
            var 3
            call
            call
            ret
           ");
    }
}
