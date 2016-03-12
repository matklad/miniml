use syntax::Ident;

pub trait Context<'a> {
    type Item;

    fn empty() -> Self;
    fn lookup(&self, name: &Ident) -> Option<&Self::Item>;
    fn push(&mut self, name: &'a Ident, value: Self::Item);
    fn pop(&mut self);
}

pub type StackContext<'a, T> = Vec<(&'a Ident, T)>;

impl<'a, T> Context<'a> for StackContext<'a, T> {
    type Item = T;

    fn empty() -> Self {
        Vec::new()
    }

    fn lookup(&self, name: &Ident) -> Option<&Self::Item> {
        self.iter().rev().find(|&&(ident, _)| ident == name).map(|&(_, ref val)| val)
    }

    fn push(&mut self, name: &'a Ident, value: Self::Item) {
        self.push((name, value));
    }

    fn pop(&mut self) {
        self.pop().unwrap();
    }
}
