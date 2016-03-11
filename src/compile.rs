use syntax::Expr;
use machine::{Frame, Name, Instruction};
use ir::{Ir, BinOp, If, Apply, Fun, LetFun, desugar};


pub fn compile(expr: &Expr) -> Frame {
    let expr = desugar(expr);
    expr.compile()
}

trait Compile {
    fn compile(&self) -> Frame;
}

impl Compile for Ir {
    fn compile(&self) -> Frame {
        match *self {
            Ir::Var(name) => vec![Instruction::Var(name)],
            Ir::IntLiteral(i) => vec![Instruction::PushInt(i)],
            Ir::BoolLiteral(b) => vec![Instruction::PushBool(b)],
            Ir::BinOp(ref op) => op.compile(),
            Ir::If(ref if_) => if_.compile(),
            Ir::Fun(ref fun) => fun.compile(),
            Ir::LetFun(ref let_fun) => let_fun.compile(),
            Ir::Apply(ref apply) => apply.compile(),
        }
    }
}

impl Compile for BinOp {
    fn compile(&self) -> Frame {
        use ir::BinOpKind::*;
        use machine::{ArithInstruction, CmpInstruction};
        let mut result = self.lhs.compile();
        result.extend(self.rhs.compile());
        result.push(match self.kind {
            Add => Instruction::ArithInstruction(ArithInstruction::Add),
            Sub => Instruction::ArithInstruction(ArithInstruction::Sub),
            Mul => Instruction::ArithInstruction(ArithInstruction::Mul),
            Div => Instruction::ArithInstruction(ArithInstruction::Div),
            Lt => Instruction::CmpInstruction(CmpInstruction::Lt),
            Eq => Instruction::CmpInstruction(CmpInstruction::Eq),
            Gt => Instruction::CmpInstruction(CmpInstruction::Gt),
        });
        result
    }
}

impl Compile for If {
    fn compile(&self) -> Frame {
        let mut result = self.cond.compile();
        result.push(Instruction::Branch(self.tru.compile(), self.fls.compile()));
        result
    }
}

fn make_closue(fun_name: Name, arg_name: Name, body: &Ir) -> Instruction {
    let mut frame = body.compile();
    frame.push(Instruction::PopEnv);
    Instruction::Closure {
        name: fun_name,
        arg: arg_name,
        frame: frame,
    }
}

impl Compile for Fun {
    fn compile(&self) -> Frame {
        vec![make_closue(self.fun_name, self.arg_name, &self.body)]
    }
}

impl Compile for LetFun {
    fn compile(&self) -> Frame {
        let fun = make_closue(self.fun_name, self.arg_name, &self.fun_body);
        let expr = make_closue(0, self.fun_name, &self.expr);
        vec![expr, fun, Instruction::Call]
    }
}

impl Compile for Apply {
    fn compile(&self) -> Frame {
        let mut result = self.fun.compile();
        result.extend(self.arg.compile());
        result.push(Instruction::Call);
        result
    }
}
