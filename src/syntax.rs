pub use self::types::Type;

type SymbolTable = Vec<String>;

pub struct Ident(IdentInner);

enum IdentInner {
    Interned(usize),
    Owned(String),
}

mod types {
    pub enum Type {
        Int,
        Bool,
        Arrow(Arrow),
    }

    pub struct Arrow(Box<Type>, Box<Type>);
}

mod exprs {
    use super::{Type, Ident};

    pub enum Expr {
        Var(Ident),
        Int(i64),
        Bool(bool),
        ArithBinOp(ArithBinOp),
        CompBinOp(CompBinOp),
        If(If),
        Fun(Fun),
        Application(Application),
    }

    pub struct ArithBinOp {
        pub kind: ArithBinOpKind,
        pub lhs: Box<Expr>,
        pub rhs: Box<Expr>,
    }

    pub enum ArithBinOpKind { Times, Plus, Minus }

    pub struct CompBinOp {
        pub kind: CompBinOpKind,
        pub lhs: Box<Expr>,
        pub rhs: Box<Expr>,
    }

    pub enum CompBinOpKind { Eq, Lt }

    pub struct If {
        pub cond: Box<Expr>,
        pub tru: Box<Expr>,
        pub fls: Box<Expr>,
    }

    pub struct Fun {
        pub argName: Ident,
        pub argType: Type,
        pub funName: Ident,
        pub retType: Type,
        pub body: Box<Expr>,
    }

    pub struct Application {
        pub fun: Box<Expr>,
        pub arg: Box<Expr>,
    }
}

