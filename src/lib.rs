mod syntax;
struct Ident(String);

struct Closure {
    arg: Ident,
    frame: Frame,
    environmnt: Environment,
}

enum Value {
    Int(i64),
    Bool(bool),
    Closure(Closure),
}

enum Instruction {
    Add, Sub, Mul,
    Eq, Lt,
    Var(Ident),
    Int(i64),
    Bool(bool),
    Closure(Ident, Ident, Frame),
}

type Frame = Vec<Instruction>;
struct Environment;
