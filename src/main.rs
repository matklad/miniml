extern crate miniml;


fn readline(ps: &str, buffer: &mut String) {
    use std::io::prelude::*;
    use std::io;

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
        format!("{:?}", miniml::parse(expr))
    })
}
