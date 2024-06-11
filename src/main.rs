#![feature(lazy_cell)]

mod error;
mod lex;
mod parser;
mod types;

fn main() {
    let mut display = String::new();
    let p = parser::parse_assign("CustomClass = {{1,2,3}, 1, 2, 3, 1.23}", 0, 0).unwrap();
    let _ = p.value(&mut display);
    println!("{}", display);
}
