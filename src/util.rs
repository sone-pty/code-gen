use std::path::{Path, PathBuf};

pub fn find_file<P: AsRef<Path>>(dir: P, filename: &str) -> PathBuf {
    let dir = dir.as_ref();
    if let Ok(rdir) = std::fs::read_dir(dir) {
        for entry in rdir {
            if let Ok(e) = entry {
                let path = e.path();
                if path.is_dir() {
                    let ret = find_file(&path, filename);
                    if ret.is_file() {
                        return ret;
                    }
                } else if path.file_name().and_then(|name| name.to_str()) == Some(filename) {
                    return path.to_path_buf();
                }
            }
        }
        PathBuf::default()
    } else {
        PathBuf::default()
    }
}

pub fn conv_col_idx(mut n: usize) -> String {
    let mut result = String::new();
    while n > 0 {
        let rem = (n - 1) % 26;
        let letter = (b'A' + rem as u8) as char;
        result.push(letter);
        n = (n - 1) / 26;
    }
    result.chars().rev().collect()
}
