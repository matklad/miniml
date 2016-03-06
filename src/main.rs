extern crate miniml;

use std::io::prelude::*;
use std::io;

fn readline(ps: &str, buffer: &mut String) {
    write!(io::stdout(), "{} ", ps).unwrap();
    io::stdout().flush().unwrap();
    io::stdin().read_line(buffer).unwrap();
}

fn repl<F: Fn(&str) -> String>(f: F) {
    let mut buffer = String::new();
    println!("Hello! Type :q to quit");
    loop {
        buffer.clear();
        readline(">", &mut buffer);
        if buffer.starts_with(":q") {
            println!("Bye!");
            return;
        }
        println!("{}", f(&buffer));
    }
}

fn main() {
    repl(|expr| {
        let expr = match miniml::parse(expr) {
            Err(e) => return format!("Parse error: {:?}", e),
            Ok(e) => e,
        };
        let t = match miniml::typecheck(&expr) {
            Err(e) => return format!("Type error: {:?}", e),
            Ok(t) => t,
        };
        format!("Type: {:?}", t)
    });
}
