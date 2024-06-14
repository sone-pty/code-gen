#![feature(lazy_cell)]

mod config;
mod error;
mod lex;
mod parser;
mod types;

fn main() {
    let mut display = String::new();
    let p = parser::parse_assign(r#" decimal = 1000000000000 "#, 0, 0).unwrap();
    let _ = p.value(&mut display);
    println!("{}", display);
}
