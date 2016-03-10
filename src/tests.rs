use syntax;
use machine::{Machine, Value};
use typecheck::typecheck;
use compile::compile;

fn assert_execs<V: Into<Value<'static>>>(expected: V, program: &str) {
    let expected = expected.into();
    let program = syntax::parse(&program).unwrap();
    typecheck(&program).unwrap();
    let program = compile(&program);
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

#[test]
fn basic() {
    assert_execs(92, "92");
    assert_execs(false, "false");
}

#[test]
fn arithmetics() {
    assert_execs(92, "10 * 5 - 10 + 100 / 10 + 3 * (10 + 4)")
}

#[test]
fn factorial() {
    assert_execs(120,
                 "(fun f(n: int): int is if n == 0 then 1 else n * f (n - 1)) 5")
}

#[test]
fn fib() {
    assert_execs(144,
                 "(fun fib(n: int): int is
                            if n == 0 then 1
                            else if n == 1 then 1
                            else fib (n - 1) + fib (n - 2)) 11");
}

#[test]
fn fix_factorial() {
    // Can't typecheck fixpoint combinator ;(
    let fix_factorial = "
((fun fix(F: (int -> int) -> (int -> int)): (int -> int) is
    (fun a(x: int): int is (F fun b(n: int): int is (x x) n))
     fun a(x: int): int is (F fun b(n: int): int is (x x) n))

fun Fact(F: (int -> int)): (int -> int) is fun i(n: int): int is
    if n == 0 then 1 else n * F (n - 1))
5
";
    let program = syntax::parse(&fix_factorial).unwrap();
    let program = compile(&program);
    let mut machine = Machine::new(&program);
    assert_eq!(machine.exec().unwrap(), Value::Int(120));
}

#[test]
fn fib_let() {
    assert_execs(144,
                 "let fun fib(n: int): int is
                      if n == 0 then 1
                      else if n == 1 then 1
                      else fib (n - 1) + fib (n - 2)
                  in fib 11");
}

#[test]
fn fix_factorial_let() {
    // Can't typecheck fixpoint combinator ;(
    let fix_factorial = "
let fun fix(F: (int -> int) -> (int -> int)): (int -> int) is
    (fun a(x: int): int is (F fun b(n: int): int is (x x) n))
     fun a(x: int): int is (F fun b(n: int): int is (x x) n)
in let fun Fact(F: (int -> int)): (int -> int) is fun i(n: int): int is
    if n == 0 then 1 else n * F (n - 1)
in (fix Fact) 5
";
    let program = syntax::parse(&fix_factorial).unwrap();
    let program = compile(&program);
    let mut machine = Machine::new(&program);
    assert_eq!(machine.exec().unwrap(), Value::Int(120));
}

#[test]
fn let_shadowing() {
    assert_execs(92,
                 "let fun f(x: int): int is x * 2
                  in let fun f(x: int): int is x + 2
                  in f 90")
}
