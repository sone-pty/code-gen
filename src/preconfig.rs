use crate::{config::SOURCE_XLSXS_DIR, error::Error};
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

#[allow(dead_code)]
pub struct PreConfigData {
    pub extra_lang_sheets: Vec<String>,
    pub ctor_begin: String,
    pub ctor_end: String,
}

impl PreConfigData {
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
    let mut ret = HashMap::new();
    cfgdir.push("CustomExportConfig");

    for entry in std::fs::read_dir(cfgdir.as_path())? {
        let path = entry?.path();
        if let Some(base_name) = path.file_name() {
            if path.extension().is_some_and(|v| v.to_str() == Some("tnl")) {
                let tnl = std::fs::read_to_string(&path)?;
                let tnl = Box::leak(Box::new(tnl));
                let obj = tnl::ObjectAccessor(Box::leak(Box::new(
                    tnl::Object::try_from(tnl.as_str()).unwrap(),
                )));
                let config = obj.attribute("preconfig")?.as_object()?;
                let extra_lang_sheets = {
                    let arr = config.attribute("extra_lang_sheets")?.as_array()?;
                    let mut data = Vec::new();
                    for i in 0..arr.0.elements.len() {
                        let t = arr.index(i)?.as_str()?;
                        data.push(t.to_string());
                    }
                    data
                };
                let ctor_begin = config.attribute("ctor_begin")?.as_str()?.to_string();
                let ctor_end = config.attribute("ctor_end")?.as_str()?.to_string();
                let base_name = base_name.to_str().ok_or::<Error>(
                    "Convert path to string failed when parsing preconfig".into(),
                )?;
                let idx = base_name.find('.').unwrap_or_default();
                ret.insert(
                    base_name[..idx].into(),
                    PreConfigData {
                        extra_lang_sheets,
                        ctor_begin,
                        ctor_end,
                    },
                );
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
