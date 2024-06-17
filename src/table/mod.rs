use crate::{config::CFG, error::Error, lex::states::nodes::value_type, types::Value};
use global_config::GlobalConfig;
use std::path::Path;
use template::Template;
use xlsx_read::{excel_file::ExcelFile, excel_table::ExcelTable};

mod global_config;
mod template;

#[allow(dead_code)]
pub trait TableCore {
    fn name(&self) -> &str;
    fn build(&self, stream: &mut dyn std::io::Write, is_server: bool) -> Result<(), Error>;
    fn load(table: &ExcelTable, name: &str) -> Result<Self, Error>
    where
        Self: Sized;
}

pub struct Table {
    core: Option<Box<dyn TableCore>>,
}

impl Table {
    pub fn load<P: AsRef<Path>>(path: P, name: &str) -> Result<Self, Error> {
        let mut excel = ExcelFile::load_from_path(path)?;
        let sheets = excel.parse_workbook()?;
        let mut core = None;

        for (flag, id) in sheets.into_iter() {
            let table = excel.parse_sheet(id)?;
            match flag.as_str() {
                "Template" => {
                    core = Some(Box::new(Template::load(&table, name)?) as _);
                }
                "GlobalConfig" => {
                    core = Some(Box::new(GlobalConfig::load(&table, name)?) as _);
                }
                _ => return Err(format!("In table {}, No valid sheet name found", name).into()),
            }
        }
        Ok(Self { core })
    }

    fn get_sheet_height(table: &ExcelTable) -> Result<usize, Error> {
        for y in CFG.row_of_start..table.height() {
            if table
                .cell_content(0, y)
                .is_some_and(|v| v.trim() == CFG.eof_flag)
            {
                return Ok(y);
            }
        }
        Err("Lack of `EOF` flag".into())
    }
}

pub struct Sheet {
    col: usize,
    row: usize,
    data: Box<[RowData]>,
}

#[allow(dead_code)]
impl Sheet {
    pub fn ty(&self, col: usize, row: usize) -> Result<Box<value_type>, Error> {
        if col < self.col && row < self.row {
            crate::parser::parse_type(self.data[row - CFG.row_of_start].value(col)?, 0, 0)
        } else {
            Err("Index was out of range".into())
        }
    }

    pub fn value(
        &self,
        col: usize,
        row: usize,
        ty: &Box<value_type>,
    ) -> Result<Box<dyn Value>, Error> {
        if col < self.col && row < self.row {
            crate::parser::parse_assign_with_type(ty, self.data[row - CFG.row_of_start].value(col)?)
        } else {
            Err("Index was out of range".into())
        }
    }

    pub fn full_iter(&self) -> SheetFullIter<'_> {
        SheetFullIter {
            view: &self.data,
            c: 0,
            r: 0,
        }
    }

    pub fn iter(&self) -> SheetIter<'_> {
        SheetIter {
            view: &self.data,
            r: 0,
        }
    }
}

pub struct SheetFullIter<'a> {
    view: &'a [RowData],
    c: usize,
    r: usize,
}

impl<'a> Iterator for SheetFullIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.view.is_empty() {
            return None;
        }
        let rmax = self.view.len();
        let cmax = self.view[0].0.len();

        if self.r < rmax && self.c < cmax {
            let Ok(v) = self.view[self.r].value(self.c) else {
                return None;
            };
            let m = (self.c + 1) % cmax;
            if m == 0 {
                self.r += 1;
            }
            self.c = m;
            Some(v)
        } else {
            None
        }
    }
}

pub struct SheetIter<'a> {
    view: &'a [RowData],
    r: usize,
}

impl<'a> Iterator for SheetIter<'a> {
    type Item = &'a RowData;

    fn next(&mut self) -> Option<Self::Item> {
        if self.r < self.view.len() {
            let v = &self.view[self.r];
            self.r += 1;
            Some(v)
        } else {
            None
        }
    }
}

#[repr(transparent)]
pub struct RowData(Box<[String]>);

impl RowData {
    pub fn value(&self, col: usize) -> Result<&str, Error> {
        if col < self.0.len() {
            Ok(self.0[col].as_str())
        } else {
            Err("Exceeded the range of the row data index".into())
        }
    }

    pub fn iter(&self) -> RowDataIter<'_> {
        RowDataIter {
            view: &self.0,
            c: 0,
        }
    }
}

pub struct RowDataIter<'a> {
    view: &'a [String],
    c: usize,
}

impl<'a> Iterator for RowDataIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.c < self.view.len() {
            let v = &self.view[self.c];
            self.c += 1;
            Some(v)
        } else {
            None
        }
    }
}
