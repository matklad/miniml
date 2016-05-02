use std::str::FromStr;

use error::ParseError;

use ast::{Ident, Type, Expr, CmpOp, CmpBinOp, ArithOp, ArithBinOp, If, Fun, Apply, Literal};

pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let tokenizer = Tokenizer::new(input);
    let mut parser = Parser::new(tokenizer);
    parser.parse()
}

struct Parser<'p> {
    tokenizer: Tokenizer<'p>
}

impl<'p> Parser<'p> {
    fn new(tokenizer: Tokenizer<'p>) -> Self {
        Parser { tokenizer: tokenizer }
    }

    fn precedence(sym: Sym) -> u8 {
        match sym {
            Sym::Eq | Sym::Lt | Sym::Gt => 3,
            Sym::Add | Sym::Sub => 2,
            Sym::Mul | Sym::Div => 1,
            _ => 255,
        }
    }

    fn max_precedence() -> u8 { 255 }

    fn parse(&mut self) -> Result<Expr, ParseError> {
        self.parse_expr(Self::max_precedence())
    }

    fn parse_expr(&mut self, precedence: u8) -> Result<Expr, ParseError> {
        let mut lhs = try!(self.parse_application());

        let mut has_comarison = false;

        while let Some(sym) = self.eat_op_with_precendence(precedence) {
            let rhs = try!(self.parse_expr(Self::precedence(sym)));
            match sym {
                Sym::Eq | Sym::Lt | Sym::Gt => {
                    let kind = match sym {
                        Sym::Eq => CmpOp::Eq,
                        Sym::Lt => CmpOp::Lt,
                        Sym::Gt => CmpOp::Gt,
                        _ => unreachable!()
                    };
                    if has_comarison {
                        return Err(self.err("Chained comparisons are not allowed"));
                    }
                    has_comarison = true;

                    lhs = CmpBinOp { kind: kind, lhs: lhs, rhs: rhs }.into();
                }

                Sym::Add | Sym::Sub | Sym::Mul | Sym::Div => {
                    let kind = match sym {
                        Sym::Add => ArithOp::Add,
                        Sym::Sub => ArithOp::Sub,
                        Sym::Mul => ArithOp::Mul,
                        Sym::Div => ArithOp::Div,
                        _ => unreachable!()
                    };

                    lhs = ArithBinOp { kind: kind, lhs: lhs, rhs: rhs }.into();
                }

                _ => unreachable!()
            }
        }

        Ok(lhs)
    }

    fn parse_application(&mut self) -> Result<Expr, ParseError> {
        let mut fun = match try!(self.parse_atom()) {
            Some(fun) => fun,
            None => return Err(self.err("Expected expression"))
        };

        while let Some(arg) = try!(self.parse_atom()) {
            fun = Apply { fun: fun, arg: arg }.into();
        }

        Ok(fun)
    }

    fn parse_atom(&mut self) -> Result<Option<Expr>, ParseError> {
        match self.tokenizer.lookahead() {
            Token::Eof | Token::Paren(Paren::Close) | Token::Sym(_) => Ok(None),
            Token::Number(n) => {
                self.tokenizer.eat_token();
                Ok(Some(Expr::Literal(Literal::Number(n))))
            }
            Token::Bool(b) => {
                self.tokenizer.eat_token();
                Ok(Some(Expr::Literal(Literal::Bool(b))))
            }
            Token::Ident(i) => {
                self.tokenizer.eat_token();
                Ok(Some(Expr::Var(Ident::from_str(i))))
            }
            Token::Paren(Paren::Open) => {
                self.tokenizer.eat_token();
                let expr = try!(self.parse());
                try!(self.expect(Token::Paren(Paren::Close), "Expected `)`"));
                Ok(Some(expr))
            }
            Token::Keyword(Keyword::If) => {
                self.tokenizer.eat_token();
                Ok(Some(try!(self.parse_if()).into()))
            }
            Token::Keyword(Keyword::Fun) => {
                self.tokenizer.eat_token();
                Ok(Some(try!(self.parse_fun()).into()))
            }
            Token::Keyword(_) => Ok(None),
            Token::Unknown => Err(self.unknown()),
        }
    }

    fn parse_if(&mut self) -> Result<If, ParseError> {
        let cond = try!(self.parse());
        try!(self.expect(Token::Keyword(Keyword::Then), "Expected `then`"));
        let tru = try!(self.parse());
        try!(self.expect(Token::Keyword(Keyword::Else), "Expected `else`"));
        let fls = try!(self.parse());
        Ok(If { cond: cond, tru: tru, fls: fls })
    }

    fn parse_fun(&mut self) -> Result<Fun, ParseError> {
        let fun_name = try!(self.parse_ident());

        try!(self.expect(Token::Paren(Paren::Open), "Expected `(`"));
        let arg_name = try!(self.parse_ident());
        try!(self.expect(Token::Sym(Sym::Colon), "Expected `:`"));
        let arg_type = try!(self.parse_type());
        try!(self.expect(Token::Paren(Paren::Close), "Expected `)`"));

        try!(self.expect(Token::Sym(Sym::Colon), "Expected `:`"));
        let fun_type = try!(self.parse_type());

        try!(self.expect(Token::Keyword(Keyword::Is), "Expected `is` before function body"));
        let body = try!(self.parse());
        Ok(Fun {
            fun_name: Ident::from_str(fun_name),
            arg_name: Ident::from_str(arg_name),
            fun_type: fun_type,
            arg_type: arg_type,
            body: body,
        })
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        let arg = try!(self.parse_atom_type());
        let mut types = vec![arg];
        while let Token::Sym(Sym::Arrow) = self.tokenizer.lookahead() {
            self.tokenizer.eat_token();
            types.push(try!(self.parse_atom_type()));
        }

        let mut result = types.pop().unwrap();
        while let Some(t) = types.pop() {
            result = Type::arrow(t, result);
        }

        Ok(result)
    }

    fn parse_atom_type(&mut self) -> Result<Type, ParseError> {
        match self.tokenizer.eat_token() {
            Token::Ident(name) if name == "int" => Ok(Type::Int),
            Token::Ident(name) if name == "bool" => Ok(Type::Bool),
            Token::Paren(Paren::Open) => {
                let inner = try!(self.parse_type());
                try!(self.expect(Token::Paren(Paren::Close), "Expected `)`"));
                Ok(inner)
            }
            _ => Err(self.err("Expected type"))
        }
    }

    fn parse_ident(&mut self) -> Result<&'p str, ParseError> {
        match self.tokenizer.eat_token() {
            Token::Ident(name) => Ok(name),
            _ => Err(self.err("Expected identifier")),
        }
    }

    fn expect(&mut self, t: Token<'p>, msg: &'static str) -> Result<(), ParseError> {
        if self.tokenizer.eat_token() == t {
            Ok(())
        } else {
            Err(self.err(msg))
        }
    }

    fn eat_op_with_precendence(&mut self, precedence: u8) -> Option<Sym> {
        match self.tokenizer.lookahead() {
            Token::Sym(op) if Self::precedence(op) < precedence => {
                self.tokenizer.eat_token();
                Some(op)
            }
            _ => None
        }
    }

    fn unknown(&self) -> ParseError {
        self.err("Unknown token")
    }

    fn err(&self, msg: &'static str) -> ParseError {
        ParseError::new(self.tokenizer.position, msg.to_owned())
    }
}


struct Tokenizer<'p> {
    position: usize,
    input: &'p str,
}

impl<'p> Tokenizer<'p> {
    fn new(input: &'p str) -> Self {
        Tokenizer { position: 0, input: input }
    }

    fn lookahead(&self) -> Token<'p> {
        let (tok, _input) = self.next();
        tok
    }

    fn eat_token(&mut self) -> Token<'p> {
        let (tok, len) = self.next();
        self.advance(len);
        self.skip_whitespace();
        tok
    }

    fn next(&self) -> (Token<'p>, usize) {
        if self.input.len() == 0 {
            return (Token::Eof, 0)
        }

        macro_rules! magic {
           ( $( ($method:ident, $ctor:ident) ),* ) => {
               $(
                   if let Some((t, l)) = self.$method() { return (Token::$ctor(t), l); }
               )*
           }
        };

        magic!(
            (eat_number, Number),
            (eat_bool, Bool),
            (eat_keyword, Keyword),
            (eat_ident, Ident),
            (eat_paren, Paren),
            (eat_sym, Sym)
        );

        return (Token::Unknown, 0);
    }

    fn eat_number(&self) -> Option<(i64, usize)> {
        //TODO: negative numbers?
        let non_digit = self.input.find(|c: char| !c.is_digit(10)).unwrap_or(self.input.len());
        if non_digit == 0 {
            None
        } else {
            let n = i64::from_str(&self.input[..non_digit]).unwrap();
            Some((n, non_digit))
        }
    }

    fn eat_bool(&self) -> Option<(bool, usize)> {
        self.dispatch(&[("true", true), ("false", false)])
    }

    fn eat_paren(&self) -> Option<(Paren, usize)> {
        self.dispatch(&[("(", Paren::Open), (")", Paren::Close)])
    }

    fn eat_ident(&self) -> Option<(&'p str, usize)> {
        let non_letter = self.input.find(|c: char| !c.is_alphabetic()).unwrap_or(self.input.len());
        if non_letter == 0 {
            None
        } else {
            Some((&self.input[..non_letter], non_letter))
        }
    }

    fn eat_sym(&self) -> Option<(Sym, usize)> {
        let table = [
        ("->", Sym::Arrow),
        ("==", Sym::Eq),
        ("<", Sym::Lt),
        (">", Sym::Gt),
        ("+", Sym::Add),
        ("-", Sym::Sub),
        ("*", Sym::Mul),
        ("/", Sym::Div),
        (":", Sym::Colon),
        ];
        self.dispatch(&table)
    }

    fn eat_keyword(&self) -> Option<(Keyword, usize)> {
        let table = [
        ("if", Keyword::If),
        ("then", Keyword::Then),
        ("else", Keyword::Else),
        ("fun", Keyword::Fun),
        ("is", Keyword::Is),
        ];
        self.dispatch(&table)
    }

    fn skip_whitespace(&mut self) {
        let non_ws = self.input.find(|c: char| !c.is_whitespace()).unwrap_or(self.input.len());
        self.advance(non_ws);
    }

    fn advance(&mut self, n: usize) {
        self.position += n;
        self.input = &self.input[n..];
    }

    fn dispatch<T: Copy>(&self, table: &[(&'static str, T)]) -> Option<(T, usize)> {
        for & (pat, val) in table {
            if self.input.starts_with(pat) {
                return Some((val, pat.len()));
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Token<'p> {
    Eof,
    Unknown,
    Number(i64),
    Bool(bool),
    Ident(&'p str),
    Paren(Paren),
    Sym(Sym),
    Keyword(Keyword),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Paren {
    Open, Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Sym {
    Eq,
    Lt,
    Gt,
    Add,
    Sub,
    Mul,
    Div,
    Colon,
    Arrow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Keyword {
    If,
    Then,
    Else,
    Fun,
    Is,
}
