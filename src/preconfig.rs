use crate::{config::SOURCE_XLSXS_DIR, error::Error};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

#[derive(Deserialize, Serialize, Debug)]
pub struct PreConfigData {
    pub extra_lang_sheets: Vec<String>,
    pub ctor_begin: String,
    pub ctor_end: String,
}

impl PreConfigData {
    pub fn new(json: &str) -> Result<Self, Error> {
        let ret = serde_json::from_str(json)?;
        Ok(ret)
    }

    pub fn exist(&self, name: &str) -> bool {
        for v in self.extra_lang_sheets.iter() {
            if v.as_str() == name {
                return true;
            }
        }
        false
    }
}

fn preconfig_handler<P: AsRef<std::path::Path>>(
    dir: P,
) -> Result<HashMap<String, PreConfigData>, Error> {
    let dirpath = dir.as_ref();
    let mut cfgdir = dirpath.to_path_buf();
    let mut json = String::with_capacity(1024);
    let mut ret = HashMap::new();
    cfgdir.push("CustomExportConfig");

    for entry in std::fs::read_dir(cfgdir.as_path())? {
        let path = entry?.path();
        if let Some(base_name) = path.file_name() {
            if path.extension().is_some_and(|v| v.to_str() == Some("json")) {
                if let Ok(mut f) = std::fs::File::open(path.as_path()) {
                    let base_name = base_name.to_str().unwrap();
                    let idx = base_name.find('.').unwrap_or_default();
                    std::io::Read::read_to_string(&mut f, &mut json)?;
                    ret.insert(
                        String::from(&base_name[..idx]),
                        PreConfigData::new(json.as_str())?,
                    );
                }
            }
        }
    }
    Ok(ret)
}

pub static PRECONFIG: LazyLock<Arc<HashMap<String, PreConfigData>>> = LazyLock::new(|| {
    match preconfig_handler(std::path::PathBuf::from(unsafe { SOURCE_XLSXS_DIR })) {
        Ok(map) => Arc::new(map),
        Err(e) => {
            use ansi_term::Colour::Red;
            eprintln!(
                "{}",
                Red.bold()
                    .paint(format!("[Error]: Read Preconfig failed: {}", e))
            );
            std::process::exit(-1);
        }
    }
});
