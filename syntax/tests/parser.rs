extern crate syntax;

use syntax::{parse, Expr};


fn assert_parses(expr: &str, ast: &str) {
    let result = parse(expr);
    assert!(result.is_ok(), "\n`{}` failed to parse:\n {:?}\n", expr, result);
    let result = format!("{:?}", result.unwrap());
    assert_eq!(result, ast);
}

fn you_shall_not_parse(expr: &str) {
    assert!(parse(expr).is_err());
}

#[test]
fn test_good_expressions() {
    assert_parses("92", "92");
    assert_parses("(92)", "92");
    assert_parses("(((92)))", "92");
    assert_parses("true", "true");
    assert_parses("false", "false");
    assert_parses("spam", "spam");
    assert_parses("1 == 1", "(== 1 1)");
    assert_parses("1 < 1 + 1", "(< 1 (+ 1 1))");
    assert_parses("1 * 2 > 1", "(> (* 1 2) 1)");
    assert_parses("(1 == 2) == 3", "(== (== 1 2) 3)");
    assert_parses("1 < (2 > 3)", "(< 1 (> 2 3))");
    assert_parses("1 + 2 * 3", "(+ 1 (* 2 3))");
    assert_parses("if 1 then 2 else if 3 then 4 else 5", "(if 1 2 (if 3 4 5))");
    assert_parses("if 1 then if 2 then 3 else 4 else 5", "(if 1 (if 2 3 4) 5)");
    assert_parses("f 92 + x y z", "(+ (f 92) ((x y) z))");
    assert_parses("1 * f 92", "(* 1 (f 92))");
    assert_parses("0 * if 1 then 2 else 3", "(* 0 (if 1 2 3))")
}

#[test]
fn test_good_fns() {
    assert_parses("fun id(x: int): int is x", "(λ id (x: int): int x)");

    assert_parses("fun id(x: (int -> int) -> bool): bool -> (int -> int) is x",
                  "(λ id (x: (int -> int) -> bool): bool -> int -> int x)");

    assert_parses("fun id(x: int): int is fun id(x: int): int is x",
                  "(λ id (x: int): int (λ id (x: int): int x))");

    assert_parses("fun factorial(n: int): int is if n == 0 then 1 else n * factorial (n - 1)",
                  "(λ factorial (n: int): int (if (== n 0) 1 (* n (factorial (- n 1)))))");

    assert_parses("1 + fun f(n:bool):bool is n + 1",
                  "(+ 1 (λ f (n: bool): bool (+ n 1)))");
}

#[test]
fn test_let_fn() {
    assert_parses("let fun f(x: int): int is 92 in f 1",
                  "(let f λ(x: int): int 92 in (f 1))")
}

#[test]
fn test_let_rec() {
    assert_parses("let rec fun a(x: int): int is b x
                   and fun b(x: int): int is a x
                   in a b 92",
                  "(letrec [(λ a (x: int): int (b x))(λ b (x: int): int (a x))] in ((a b) 92))")
}
#[test]
fn test_bad_expressions() {
    you_shall_not_parse("((92)");
    you_shall_not_parse("1 == 1 == 1");
    you_shall_not_parse("1 < 1 > 1");
}

#[test]
fn test_expr_is_small() {
    let size = std::mem::size_of::<Expr>();
    assert!(size <= 32, "Expr size is to large: {}", size);
}
