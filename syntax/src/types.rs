use std::fmt;
use std::rc::Rc;

#[derive(PartialEq, Eq, Clone)]
pub enum Type {
    Int,
    Bool,
    Arrow(Rc<Type>, Rc<Type>),
}

impl Type {
    pub fn arrow(arg: &Type, ret: &Type) -> Type {
        Type::Arrow(Rc::new(arg.clone()), Rc::new(ret.clone()))
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Type::*;
        match *self {
            Int => f.write_str("int"),
            Bool => f.write_str("bool"),
            Arrow(ref l, ref r) => {
                match **l {
                    Arrow(..) => write!(f, "({:?}) -> {:?}", l, r),
                    _ => write!(f, "{:?} -> {:?}", l, r),
                }
            }
        }
    }
}
