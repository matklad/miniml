use std::fmt;

#[derive(PartialEq, Eq, Hash)]
pub struct Ident(String);

impl Ident {
    pub fn from_str(name: &str) -> Ident {
        Ident(name.to_owned())
    }
}

impl AsRef<str> for Ident {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        assert_eq!(Ident::from_str("Hello").as_ref(), "Hello");
    }
}
