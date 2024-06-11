use std::fs::File;
use std::io::Write;
use vnlex::syntax_builder;

fn main() {
    let custom_type = "crate::lex::CData<'a>";
    match syntax_builder::parse("mod lex;", None, "".as_ref(), custom_type) {
        Ok(desc) => {
            for p in desc.mods() {
                println!("cargo:rerun-if-changed={}", p.display());
            }
            let mut file = File::create("src/lex/states.rs").expect("create file failed.");
            write!(&mut file, "{}", desc).expect("write file failed.");
        }
        Err(e) => panic!("{}", e),
    }
}
