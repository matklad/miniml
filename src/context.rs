use syntax::Ident;
use typecheck::Type;

pub struct TypeContext<'a>(Vec<(&'a Ident, Type)>);

impl<'a> TypeContext<'a> {
    pub fn empty() -> Self {
        TypeContext(Vec::new())
    }

    pub fn lookup(&self, name: &Ident) -> Option<&Type> {
        self.0.iter().rev().find(|&&(ident, _)| ident == name).map(|&(_, ref val)| val)
    }

    pub fn with_bindings<R, F, I>(&mut self, bindings: I, f: F) -> R
        where F: FnOnce(&mut TypeContext<'a>) -> R,
              I: IntoIterator<Item = (&'a Ident, Type)>
    {
        let old_bindings = self.0.len();
        self.0.extend(bindings.into_iter());
        let result = f(self);
        self.0.truncate(old_bindings);
        result
    }
}
