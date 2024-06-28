use crate::Visitor;
use std::fmt::Write;


pub struct Text {
    pub output: String,
    indent: usize,
}

impl Text {

    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
        }
    }

    pub fn format(&mut self, obj: &crate::Object) {
        for (k, v) in obj.attributes.iter() {
            self.output.push_str(&k.value);
            self.output.push_str(": ");
            v.accept(self);
            self.new_line();
        }

        for e in obj.base.elements.iter() {
            e.accept(self);
            self.new_line();
        }
    }

    fn args(&mut self, args: std::fmt::Arguments) {
        let _ = write!(&mut self.output, "{args}");
    }

    fn new_line(&mut self) {
        self.output.push_str("\n");
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
    }
}

impl<'a> Visitor<'a> for Text {
    fn object(&mut self, val: &crate::Object<'a>) {
        if !val.name.as_ref().is_empty() {
            self.output.push_str("@");
            if let Some(ref ns) = val.ns {
                self.args(format_args!("{}:{} {{", ns, val.name));
            } else {
                self.args(format_args!("{} {{", val.name));
            }
        } else {
            self.output.push_str("{");
        }

        if val.attributes.is_empty() && val.base.elements.is_empty() {
            self.output.push_str("}");
        } else {
            self.indent += 1;

            for (k, v) in val.attributes.iter() {
                self.new_line();
                self.output.push_str(&k.value);
                self.output.push_str(": ");
                v.accept(self);
            }

            for e in val.base.elements.iter() {
                self.new_line();
                e.accept(self);
            }

            self.indent -= 1;
            self.new_line();
            self.output.push_str("}");
        }
    }

    fn array(&mut self, val: &crate::Array<'a>) {
        if val.elements.is_empty() {
            self.output.push_str("[]");
        } else {
            self.output.push_str("[");
            self.indent += 1;
            for e in val.elements.iter() {
                self.new_line();
                e.accept(self);
            }

            self.indent -= 1;
            self.new_line();
            self.output.push_str("]");
        }
    }

    fn null(&mut self, _: &crate::Null) {
        self.output.push_str("null");
    }

    fn bool(&mut self, val: &crate::Boolean) {
        self.output.push_str(if val.value { "true" } else { "false" });
    }

    fn int(&mut self, val: &crate::Integer) {
        if val.minus {
            self.output.push_str("-");
        }
        self.args(format_args!("{}", val.value));
    }

    fn float(&mut self, val: &crate::Float) {
        self.args(format_args!("{}", val.value));
    }

    fn string(&mut self, val: &crate::String<'a>) {
        self.args(format_args!("\"{}\"", val.value.escape_default()));
    }

    fn ident(&mut self, val: &crate::Ident<'a>) {
        self.output.push_str(&val.value);
    }
}