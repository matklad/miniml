use std::collections::HashMap;
use syntax::{self, Expr, Literal};
use machine::{Frame, Name, Instruction};


pub fn compile(expr: &Expr) -> Frame {
    let mut name_table: HashMap<&str, Name> = HashMap::new();
    let mut get_name = |name| {
        match name_table.get(name) {
            Some(&id) => id,
            None => {
                let id = name_table.len();
                name_table.insert(name, id);
                id
            }
        }
    };
    expr.compile(&mut get_name)
}

trait Compile {
    fn compile<'a, F: FnMut(&'a str) -> usize>(&'a self, get_name: &mut F) -> Frame;
}

impl Compile for Expr {
    fn compile<'a, F: FnMut(&'a str) -> usize>(&'a self, get_name: &mut F) -> Frame {
        match *self {
            Expr::Var(ref name) => vec![Instruction::Var(get_name(name.as_ref()))],
            Expr::Literal(Literal::Number(i)) => vec![Instruction::PushInt(i)],
            Expr::Literal(Literal::Bool(b)) => vec![Instruction::PushBool(b)],
            Expr::ArithBinOp(ref op) => op.compile(get_name),
            Expr::CmpBinOp(ref op) => op.compile(get_name),
            Expr::If(ref if_) => if_.compile(get_name),
            Expr::Fun(ref fun_) => fun_.compile(get_name),
            Expr::Apply(ref apply) => apply.compile(get_name),
        }
    }
}

impl<I: CompileInstruction> Compile for syntax::BinOp<I> {
    fn compile<'a, F: FnMut(&'a str) -> usize>(&'a self, get_name: &mut F) -> Frame {
        let mut result = self.lhs.compile(get_name);
        result.extend(self.rhs.compile(get_name));
        result.push(self.kind.compile());
        result
    }
}

impl Compile for syntax::If {
    fn compile<'a, F: FnMut(&'a str) -> usize>(&'a self, get_name: &mut F) -> Frame {
        let mut result = self.cond.compile(get_name);
        result.push(Instruction::Branch(self.tru.compile(get_name), self.fls.compile(get_name)));
        result
    }
}

impl Compile for syntax::Fun {
    fn compile<'a, F: FnMut(&'a str) -> usize>(&'a self, get_name: &mut F) -> Frame {
        let mut frame = self.body.compile(get_name);
        frame.push(Instruction::PopEnv);
        vec![Instruction::Closure {
                 name: get_name(self.name.as_ref()),
                 arg: get_name(self.arg_name.as_ref()),
                 frame: frame,
             }]
    }
}

impl Compile for syntax::Apply {
    fn compile<'a, F: FnMut(&'a str) -> usize>(&'a self, get_name: &mut F) -> Frame {
        let mut result = self.fun.compile(get_name);
        result.extend(self.arg.compile(get_name));
        result.push(Instruction::Call);
        result
    }
}

trait CompileInstruction {
    fn compile(&self) -> Instruction;
}

impl CompileInstruction for syntax::ArithOp {
    fn compile(&self) -> Instruction {
        use syntax::ArithOp;
        use machine::ArithInstruction;

        Instruction::ArithInstruction(match *self {
            ArithOp::Add => ArithInstruction::Add,
            ArithOp::Sub => ArithInstruction::Sub,
            ArithOp::Mul => ArithInstruction::Mul,
            ArithOp::Div => ArithInstruction::Div,
        })
    }
}

impl CompileInstruction for syntax::CmpOp {
    fn compile(&self) -> Instruction {
        use syntax::CmpOp;
        use machine::CmpInstruction;

        Instruction::CmpInstruction(match *self {
            CmpOp::Lt => CmpInstruction::Lt,
            CmpOp::Eq => CmpInstruction::Eq,
            CmpOp::Gt => CmpInstruction::Gt,
        })
    }
}
