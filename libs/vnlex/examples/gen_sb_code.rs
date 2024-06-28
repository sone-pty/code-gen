use vnlex::syntax_builder;



fn main() {
    match syntax_builder::parse("mod main;", None, "".as_ref(), "()") {
        Ok(desc) => {
            println!("{desc}");
        }
        Err(e) => eprintln!("{e}"),
    }
}