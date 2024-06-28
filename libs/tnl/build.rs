use std::fs::File;
use std::io::Write;
use vnlex::syntax_builder;

fn main() {
    match syntax_builder::parse("mod data;", None, "".as_ref(), "()") {
        Ok(desc) => {
            for p in desc.mods() {
                println!("cargo:rerun-if-changed={}", p.display());
            }
            let mut file = File::create("src/states.rs").expect("create file failed.");
            write!(&mut file, "{}", desc).expect("write file failed.");
        }
        Err(e) => panic!("{}", e),
    }
}