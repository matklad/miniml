use syntax::Ident;

pub trait Context<'a, T> {
    fn empty() -> Self;
    fn lookup(&self, name: &Ident) -> Option<&T>;
    fn push(&mut self, name: &'a Ident, value: T);
    fn pop(&mut self);
}

pub type StackContext<'a, T> = Vec<(&'a Ident, T)>;

impl<'a, T> Context<'a, T> for StackContext<'a, T> {
    fn empty() -> Self {
        Vec::new()
    }

    fn lookup(&self, name: &Ident) -> Option<&T> {
        self.iter().rev().find(|&&(ident, _)| ident == name).map(|&(_, ref val)| val)
    }

    fn push(&mut self, name: &'a Ident, value: T) {
        self.push((name, value));
    }

    fn pop(&mut self) {
        self.pop().unwrap();
    }
}
