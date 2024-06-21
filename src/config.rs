#![allow(dead_code)]

use crate::error::Error;
use std::{collections::HashSet, sync::LazyLock};

pub struct Config {
    pub row_of_start: usize,
    pub row_of_default: usize,
    pub row_of_comment: usize,
    pub row_of_ident: usize,
    pub row_of_type: usize,
    pub row_of_fk: usize,
    pub row_of_enum: usize,
    pub cell_of_defkey: (usize, usize), // (col, row)
    pub rows_to_switch: usize,
    pub eof_flag: &'static str,
    pub source_table_suffix: &'static str,
    pub dest_code_suffix: &'static str,
    pub ref_text_suffix: &'static str,
    pub language_file_name: &'static str,
    pub language_file_suffix: &'static str,
    pub line_end_flag: &'static str,
    pub col_of_enum_ident: usize,
    pub col_of_enum_val: usize,
    pub col_of_enum_desc: usize,
    pub file_banner: &'static str,
    pub language_xlsx_name: &'static str,
    pub ref_start_num: i32,
}

impl Config {
    pub fn load(tnl: &'static tnl::Object) -> Result<Self, Error> {
        let obj = tnl::ObjectAccessor(tnl);
        let config = obj.attribute("config")?.as_object()?;
        Ok(Self {
            row_of_start: config.attribute("row_of_start")?.as_u32()? as _,
            row_of_default: config.attribute("row_of_default")?.as_u32()? as _,
            row_of_comment: config.attribute("row_of_comment")?.as_u32()? as _,
            row_of_ident: config.attribute("row_of_ident")?.as_u32()? as _,
            row_of_type: config.attribute("row_of_type")?.as_u32()? as _,
            row_of_fk: config.attribute("row_of_fk")?.as_u32()? as _,
            row_of_enum: config.attribute("row_of_enum")?.as_u32()? as _,
            rows_to_switch: config.attribute("rows_to_switch")?.as_u32()? as _,
            col_of_enum_desc: config.attribute("col_of_enum_desc")?.as_u32()? as _,
            col_of_enum_ident: config.attribute("col_of_enum_ident")?.as_u32()? as _,
            col_of_enum_val: config.attribute("col_of_enum_val")?.as_u32()? as _,
            cell_of_defkey: {
                let cell = config.attribute("cell_of_defkey")?.as_object()?;
                (
                    cell.attribute("col")?.as_u32()? as _,
                    cell.attribute("row")?.as_u32()? as _,
                )
            },
            source_table_suffix: config.attribute("source_table_suffix")?.as_str()?,
            eof_flag: config.attribute("eof_flag")?.as_str()?,
            dest_code_suffix: config.attribute("dest_code_suffix")?.as_str()?,
            ref_text_suffix: config.attribute("ref_text_suffix")?.as_str()?,
            language_file_name: config.attribute("language_file_name")?.as_str()?,
            language_file_suffix: config.attribute("language_file_suffix")?.as_str()?,
            line_end_flag: config.attribute("line_end_flag")?.as_str()?,
            file_banner: config.attribute("file_banner")?.as_str()?,
            language_xlsx_name: config.attribute("language_xlsx_name")?.as_str()?,
            ref_start_num: config.attribute("ref_start_num")?.as_i32()?,
        })
    }
}

pub static CFG: LazyLock<Config> = LazyLock::new(|| {
    let tnl = std::fs::read_to_string("config.tnl").unwrap();
    let tnl = Box::leak(Box::new(tnl));
    Config::load(Box::leak(Box::new(
        tnl::Object::try_from(tnl.as_str()).unwrap(),
    )))
    .unwrap()
});

pub static mut OUTPUT_SCRIPT_CODE_DIR: &'static str = "ExportScripts/";
pub static mut OUTPUT_ENUM_CODE_DIR: &'static str = "ConfigExportEnum/";
pub static mut OUTPUT_SERVER_SCRIPT_CODE_DIR: &'static str = "ServerExportScripts/";
pub static mut OUTPUT_SERVER_ENUM_CODE_DIR: &'static str = "ServerConfigExportEnum/";
pub static mut SOURCE_XLSXS_DIR: &'static str = "D:/config";
pub static mut REF_TEXT_DIR: &'static str = "ConfigRefNameMapping/";
pub static mut CONFIG_COLLECTION_PATH: &'static str = "";
pub static mut LANG_OUTPUT_DIR: &'static str = "Data/";
pub static TABLE_XLSX_FILTER: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut ret = HashSet::<&'static str>::default();
    ret.insert("NameCore_CN.xlsx");
    ret.insert("DeadCharacter.xlsx");
    ret.insert("InscribedCharacter.xlsx");
    ret.insert("LString.xlsx");
    ret.insert("Shell");
    ret.insert("CustomExportConfig");
    ret
});
pub static ENUM_FLAGS_FILTER: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut ret = HashSet::<&'static str>::default();
    ret.insert("Inherit");
    ret.insert("Archive, Inherit");
    ret.insert("Archive, Readonly");
    ret.insert("Readonly, Inherit");
    ret.insert("Archive, Readonly, Inherit");
    ret.insert("Archive");
    ret.insert("Readonly");
    ret.insert("0");
    ret.insert("1");
    ret
});
