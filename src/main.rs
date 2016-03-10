extern crate miniml;

use std::io::prelude::*;
use std::fs::File;
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

fn execute(expr: &str) -> String {
    let expr = match miniml::parse(expr) {
        Err(e) => return format!("Parse error: {:?}", e),
        Ok(e) => e,
    };
    let t = match miniml::typecheck(&expr) {
        Err(e) => return format!("Type error: {:?}", e),
        Ok(t) => t,
    };
    let program = miniml::compile(&expr);
    let mut machine = miniml::Machine::new(&program);
    let result = match machine.exec() {
        Err(e) => return format!("{}", e.message),
        Ok(x) => x,
    };
    format!("{}", result)

}

fn start_repl() {
    repl(execute);
}

fn exec_file(path: &str) {
    let mut buffer = String::new();
    let mut file = File::open(path).unwrap();
    file.read_to_string(&mut buffer).unwrap();
    let result = execute(&buffer);
    println!("{}", result);
}

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    if let Some(file) = args.next() {
        exec_file(&file)
    } else {
        start_repl()
    }
}
