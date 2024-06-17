use std::{
    borrow::{Borrow, Cow},
    path::Path,
};

use xlsx_read::{excel_file::ExcelFile, excel_table::ExcelTable};

use crate::{config::CFG, error::Error};

pub struct Table<'a> {
    template: Sheet<'a>,
    enums: Vec<Sheet<'a>>,
}

impl<'a> Table<'a> {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut excel = ExcelFile::load_from_path(path)?;
        let sheets = excel.parse_workbook()?;
        let mut template = None;

        for (flag, id) in sheets.into_iter() {
            let table = excel.parse_sheet(id)?;

            match flag.as_str() {
                "Template" => {
                    let row = Self::get_sheet_height(&table)?;
                    let col = table.width();
                    let data = unsafe {
                        let mut raw = Box::<[RowData]>::new_uninit_slice(row);
                        for r in 0..row {
                            let mut row_data = Box::<[Cow<'_, String>]>::new_uninit_slice(col);
                            for c in 0..col {
                                row_data[c].as_mut_ptr().write(Cow::Owned(
                                    table
                                        .cell_content(c, r)
                                        .ok_or::<Error>("load table cell failed".into())?
                                        .into(),
                                ));
                            }
                            raw[r].as_mut_ptr().write(RowData(row_data.assume_init()));
                        }
                        raw.assume_init()
                    };
                    template = Some(Sheet { col, row, data });
                }
                _ => {}
            }
        }

        if template.is_none() {
            Err("lack of `Template` sheet".into())
        } else {
            Ok(Self {
                template: unsafe { template.unwrap_unchecked() },
                enums: vec![],
            })
        }
    }

    fn get_sheet_height(table: &ExcelTable) -> Result<usize, Error> {
        for y in CFG.row_of_start..table.height() {
            if table.cell_content(0, y).is_some_and(|v| v == CFG.eof_flag) {
                return Ok(y - 1);
            }
        }
        Err("lack of `EOF` flag".into())
    }
}

pub struct Sheet<'a> {
    col: usize,
    row: usize,
    data: Box<[RowData<'a>]>,
}

#[repr(transparent)]
pub struct RowData<'a>(Box<[Cow<'a, String>]>);
