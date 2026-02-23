use std::io::{self, Read};

mod function;

fn main() {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("failed to read from stdin");
    match function::run(input) {
        Ok(output) => println!("{output}"),
        Err(err) => eprintln!("error: {err}"),
    }
}
