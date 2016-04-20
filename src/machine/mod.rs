use std::collections::HashMap;
pub use self::program::{Frame, Instruction, Name, ArithInstruction, CmpInstruction};
pub use self::value::{Value, Closure};

mod value;
mod program;

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
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
    program: &'p Frame,
    storage: Vec<Env<'p>>,
    values: Vec<Value<'p>>,
    environments: Vec<Env<'p>>,
    activations: Vec<Activation<'p>>,
}

type Env<'p> = HashMap<Name, Value<'p>>;

impl<'p> Machine<'p> {
    pub fn new(program: &'p Frame) -> Self {
        Machine {
            program: program,
            storage: vec![],
            values: vec![],
            environments: vec![Env::new()],
            activations: vec![program],
        }
    }

    pub fn exec(&mut self) -> Result<Value<'p>> {
        let mut step = 0;
        while let Some(inst) = self.fetch_instruction() {
            step += 1;
            try!(inst.exec(self));
            if step % 92 == 0 {
                self.gc()
            }
        }
        self.pop_value().and_then(|result| {
            if !self.values.is_empty() {
                return Err(fatal_error("more then one value on stack left"));
            }
            Ok(result)
        })
    }

    fn fetch_instruction(&mut self) -> Option<&'p Instruction> {
        self.activations.pop().and_then(|act| {
            act.split_first().map(|(inst, act)| {
                if !act.is_empty() {
                    self.activations.push(act);
                }
                inst
            })
        })
    }

    fn switch_frame(&mut self, frame: &'p [Instruction]) {
        self.activations.push(frame)
    }

    fn push_int(&mut self, value: i64) {
        self.push_value(Value::Int(value))
    }

    fn push_bool(&mut self, value: bool) {
        self.push_value(Value::Bool(value))
    }

    fn push_value(&mut self, value: Value<'p>) {
        self.values.push(value)
    }

    fn pop_int(&mut self) -> Result<i64> {
        self.pop_value().and_then(|v| v.into_int())
    }

    fn pop_bool(&mut self) -> Result<bool> {
        self.pop_value().and_then(|v| v.into_bool())
    }

    fn pop_closure(&mut self) -> Result<Closure<'p>> {
        self.pop_value().and_then(|v| v.into_closure())
    }

    fn pop_value(&mut self) -> Result<Value<'p>> {
        self.values
            .pop()
            .ok_or(fatal_error("empty stack"))
    }

    fn lookup(&mut self, name: Name) -> Result<Value<'p>> {
        self.current_env().get(&name).cloned().ok_or(fatal_error("undefined variable"))
    }

    fn current_env(&self) -> &Env<'p> {
        self.environments.last().unwrap()
    }

    fn pop_env(&mut self) -> Result<()> {
        if self.environments.len() == 0 {
            return Err(fatal_error("no environment"));
        }
        self.environments.pop();
        Ok(())
    }

    fn gc(&mut self) {
        let mut moved: HashMap<usize, usize> = HashMap::new();

        let mut initial_work: Vec<&mut Value<'p>> = self.values.iter_mut().collect();
        initial_work.extend(self.environments.iter_mut().flat_map(|env|
            env.iter_mut().map(|(_key, value)| value)
        ));

        let mut new_storage = collect(initial_work, &mut moved, &mut self.storage, 0);
        let mut done = 0;
        loop {
            let move_index = new_storage.len();
            let wave = {
                let work = new_storage[done..].iter_mut().flat_map(|env|
                    env.iter_mut().map(|(_key, value)| value)
                ).collect();
                collect(work, &mut moved, &mut self.storage, move_index)
            };

            if wave.is_empty() {
                break;
            }
            done = new_storage.len();
            new_storage.extend(wave.into_iter());
        }

        assert!(new_storage.len() <= self.storage.len());

        self.storage = new_storage
    }
}

fn collect<'p>(work: Vec<&mut Value<'p>>,
               move_map: &mut HashMap<usize, usize>,
               old_envs: &mut [Env<'p>],
               start_index: usize,
) -> Vec<Env<'p>> {
    let mut wave: Vec<Env<'p>> = vec![];
    for value in work {
        if let Value::Closure(ref mut closure) = *value {
            if let Some(&new_index) = move_map.get(&closure.env) {
                closure.env = new_index
            } else {
                let new_index = start_index + wave.len();
                move_map.insert(closure.env, new_index);

                let mut new_env = HashMap::new();
                ::std::mem::swap(&mut new_env, &mut old_envs[closure.env]);

                closure.env = new_index;
                wave.push(new_env);
            }
        }
    }

    wave
}

trait Exec {
    fn exec<'p>(&'p self, state: &mut Machine<'p>) -> Result<()>;
}

impl Exec for Instruction {
    fn exec<'p>(&'p self, machine: &mut Machine<'p>) -> Result<()> {
        use self::program::Instruction::*;

        match *self {
            ArithInstruction(ref inst) => try!(inst.exec(machine)),
            CmpInstruction(ref inst) => try!(inst.exec(machine)),
            PushInt(i) => machine.push_int(i),
            PushBool(b) => machine.push_bool(b),
            Branch(ref tru, ref fls) => {
                let jump = if try!(machine.pop_bool()) {
                    tru
                } else {
                    fls
                };
                machine.switch_frame(jump);
            }
            Var(name) => {
                let value = try!(machine.lookup(name));
                machine.push_value(value);
            }
            Closure { name, arg, ref frame } => {
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
                machine.switch_frame(frame);
            }
            PopEnv => try!(machine.pop_env()),
        }
        Ok(())
    }
}

impl Exec for ArithInstruction {
    fn exec<'p>(&'p self, machine: &mut Machine<'p>) -> Result<()> {
        use self::program::ArithInstruction::*;
        let op2 = try!(machine.pop_int());
        let op1 = try!(machine.pop_int());
        let ret = match *self {
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
    fn exec<'p>(&'p self, machine: &mut Machine<'p>) -> Result<()> {
        use self::program::CmpInstruction::*;
        let op2 = try!(machine.pop_int());
        let op1 = try!(machine.pop_int());
        let ret = match *self {
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
    use super::*;

    fn push_instr<V: Into<Value<'static>>>(v: V) -> Instruction {
        match v.into() {
            Value::Int(i) => Instruction::PushInt(i),
            Value::Bool(b) => Instruction::PushBool(b),
            _ => unreachable!(),
        }
    }

    macro_rules! secd {
        ( (do $($tt:tt)*) ) => { secd![$($tt)*] };
        ( $( $tt:tt )* ) => { vec![ $( secd_instr!($tt) ),* ] };
    }

    macro_rules! secd_instr {
        ( call ) => { Instruction::Call };
        ( ret ) => { Instruction::PopEnv };
        ( add ) => { Instruction::ArithInstruction(ArithInstruction::Add) };
        ( sub ) => { Instruction::ArithInstruction(ArithInstruction::Sub) };
        ( mul ) => { Instruction::ArithInstruction(ArithInstruction::Mul) };
        ( div ) => { Instruction::ArithInstruction(ArithInstruction::Div) };
        ( lt ) => { Instruction::CmpInstruction(CmpInstruction::Lt) };
        ( eq ) => { Instruction::CmpInstruction(CmpInstruction::Eq) };
        ( gt ) => { Instruction::CmpInstruction(CmpInstruction::Gt) };
        ( (push $e:expr) ) => { push_instr($e) };
        ( (var $e:expr) ) => { Instruction::Var($e) };
        ( (branch $tru:tt $fls:tt) ) => {
            Instruction::Branch(secd![$tru], secd![$fls])
        };
        ( (clos ($name:expr, $arg:expr) $body:tt) ) => {
            Instruction::Closure {
                name: $name,
                arg: $arg,
                frame: secd![$body],
            }
        };
    }

    fn assert_execs<V: Into<Value<'static>>>(expected: V, program: Frame) {
        let expected = expected.into();
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

    fn assert_fails(expected_message: &str, program: Frame) {
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
        assert_execs(92, secd![ (push 92) ]);
        assert_fails("Fatal: empty stack :(", secd![]);
        assert_fails("Fatal: more then one value on stack left :(",
                     secd![(push 1)
                           (push 2)]);
    }

    #[test]
    fn arith() {
        assert_execs(92, secd![(push 90) (push 2) add]);
        assert_execs(92, secd![(push 94) (push 2) sub]);
        assert_execs(92, secd![(push 46) (push 2) mul]);
        assert_execs(92, secd![(push 184) (push 2) div]);
        assert_fails("Division by zero", secd![(push 1) (push 0) div]);
        assert_fails("Fatal: empty stack :(", secd![add]);
        assert_fails("Fatal: runtime type error :(",
                     secd![(push 1) (push true) add]);
    }
    #[test]
    fn cmp() {
        assert_execs(false, secd![(push 92) (push 62) lt]);
        assert_execs(true, secd![(push 92) (push 62) gt]);
        assert_execs(false, secd![(push 1) (push 2) eq]);
        assert_execs(true, secd![(push 2) (push 2) eq]);

        assert_fails("Fatal: runtime type error :(",
                     secd![(push 1) (push true) eq]);
        assert_fails("Fatal: runtime type error :(",
                     secd![(push true) (push false) eq]);
    }
    #[test]
    fn branch() {
        assert_execs(92,
                     secd![(push true)
                           (branch
                               (push 92)
                               (push 62))]);

        assert_execs(62,
                     secd![(push false)
                           (branch
                               (push 92)
                               (push 62))]);

        assert_execs(92,
                     secd![(push true)
                           (branch
                               (push 41)
                               (push 51))
                           (push false)
                           (branch
                               (push 41)
                               (push 51))
                           add]);
        assert_fails("Fatal: runtime type error :(",
                     secd![(push 92)
                           (branch
                               (push true)
                               (push false))]);
    }

    #[test]
    fn vars() {
        assert_execs(92,
                     secd![(clos (0, 1) (var 1))
                               (push 92)
                               call]);

        assert_fails("Fatal: undefined variable :(", secd![(var 92)]);
    }

    #[test]
    fn factorial() {
        let factorial = secd![
            (clos (0, 1) (do
                (push 0)
                (var 1)
                eq
                (branch
                    (push 1)
                    (do
                        (var 1)
                        (var 0)
                        (var 1)
                        (push 1)
                        sub
                        call
                        mul))
                ret))
            (push 5)
            call
        ];
        assert_execs(120, factorial);
    }

    #[test]
    fn hof() {
        let apply_twice = secd![
            (clos (0, 1) (do
                (clos (2, 3) (do
                    (var 1)
                    (var 1)
                    (var 3)
                    call
                    call
                    ret))
                ret))
            (clos (0, 1) (do
                (var 1)
                (var 1)
                add
                ret))
            call
            (push 23)
            call
        ];

        assert_execs(92, apply_twice);
    }
}
