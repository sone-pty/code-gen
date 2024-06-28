use std::collections::HashMap;

use vnutil::io::{WriteExt as _, WriteTo};

pub struct StringLibrary<'a> {
    indices: HashMap<&'a str, usize>,
    strings: Vec<&'a str>,
}

impl Default for StringLibrary<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StringLibrary<'a> {
    pub fn new() -> Self {
        let mut indices = HashMap::new();
        indices.insert("", 0);
        Self {
            indices,
            strings: vec![""],
        }
    }

    pub fn get_index(&mut self, string: &'a str) -> usize {
        use std::collections::hash_map::Entry;
        match self.indices.entry(string) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                let index = self.strings.len();
                self.strings.push(string);
                e.insert(index);
                index
            }
        }
    }

    pub fn get(&self, index: usize) -> Option<&'a str> {
        self.strings.get(index).map(|t| *t)
    }
}

impl WriteTo for StringLibrary<'_> {
    fn write_to<W: std::io::Write + ?Sized>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_compressed_usize(self.strings.len() - 1, usize::MAX - 1)?;
        for s in unsafe { self.strings.get_unchecked(1..) } {
            s.write_to(w)?;
        }
        Ok(())
    }
}