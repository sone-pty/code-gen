use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, Write},
};

use ansi_term::Colour::Red;
use xlsx_read::excel_table::ExcelTable;

use crate::{
    config::{CFG, REF_TEXT_DIR},
    error::Error,
    util,
};

use super::{RowData, Sheet, Table, TableCore};

pub struct Template<'a> {
    name: String,
    pub(crate) enums: Option<Enums<'a>>,
    main: Sheet,
    refs: (HashMap<String, i32>, i32), // (mappings, max_num)
}

impl Template<'_> {
    fn load_template(table: &ExcelTable, name: &str) -> Result<Self, Error> {
        let (mut refs, mut max_ref_num, mut ref_file) = Self::load_refs(name)?;
        let row = Table::get_sheet_height(table)?;
        let col = table.width();
        let mut table_refs_set = HashSet::new();
        let init = max_ref_num == CFG.ref_start_num - 1;

        // build row data
        let data = unsafe {
            let mut raw = Box::<[RowData]>::new_uninit_slice(row);
            for r in 0..row {
                let mut row_data = Box::<[String]>::new_uninit_slice(col);
                for c in 0..col {
                    row_data[c]
                        .as_mut_ptr()
                        .write(table.cell_content(c, r).unwrap_or("").trim().into());
                }
                raw[r].as_mut_ptr().write(RowData(row_data.assume_init()));
            }
            raw.assume_init()
        };

        // extra None ref value
        if init {
            max_ref_num += 1;
            ref_file.write_fmt(format_args!(
                "{}{}{}{}",
                "None", CFG.line_end_flag, "-1", CFG.line_end_flag
            ))?;
            refs.insert("None".into(), -1);
        }

        // check ref
        for r in CFG.row_of_start..row {
            table_refs_set.insert(data[r].value(0)?);
            let ref_id = table
                .cell_content(0, r)
                .ok_or::<Error>(
                    format!(
                        "Load table cell failed: table name = {}, in cell ({}, {})",
                        name,
                        r + 1,
                        util::conv_col_idx(1),
                    )
                    .into(),
                )?
                .trim();

            if init {
                // new ref file
                max_ref_num += 1;
                ref_file.write_fmt(format_args!("{}{}", ref_id, CFG.line_end_flag))?;
                ref_file.write_fmt(format_args!("{}{}", max_ref_num, CFG.line_end_flag))?;
                refs.insert(ref_id.into(), max_ref_num);
            } else {
                // append ref file
                match refs.entry(ref_id.into()) {
                    std::collections::hash_map::Entry::Vacant(e) => {
                        max_ref_num += 1;
                        ref_file.write_fmt(format_args!("{}{}", ref_id, CFG.line_end_flag))?;
                        ref_file.write_fmt(format_args!("{}{}", max_ref_num, CFG.line_end_flag))?;
                        e.insert(max_ref_num);
                    }
                    _ => {}
                }
            }
        }

        // flush
        ref_file.flush()?;

        // Check if the table contains the data row in ref file
        for (id, _) in refs.iter().filter(|v| v.0 != "None") {
            if !table_refs_set.contains(id.as_str()) {
                eprintln!(
                    "{}",
                    Red.bold().paint(format!(
                        "[Error]: In the table {}.xlsx, lack of row: {} which is found in ref.txt",
                        name, id,
                    ))
                );
            }
        }

        Ok(Self {
            main: Sheet { col, row, data },
            enums: None,
            refs: (refs, max_ref_num),
            name: name.into(),
        })
    }

    fn load_refs(name: &str) -> Result<(HashMap<String, i32>, i32, File), Error> {
        let path = format!(
            "{}/{}.{}",
            unsafe { REF_TEXT_DIR },
            name,
            CFG.ref_text_suffix,
        );
        let mut max_num = CFG.ref_start_num - 1;
        let mut data = HashMap::new();

        match File::open(&path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut key = String::new();
                let mut ctl = false;

                for v in std::io::BufRead::lines(reader) {
                    if let Ok(v) = v {
                        let v: String = v.trim().into(); // allcoate string here
                        if v.is_empty() {
                            continue;
                        }

                        if !ctl {
                            key = v;
                        } else {
                            let num = v.parse()?;
                            max_num = std::cmp::max(max_num, num);
                            data.insert(key, num);
                            key = String::new(); // in-expensive
                        }
                    }
                    ctl = !ctl;
                }
                Ok((
                    data,
                    max_num,
                    std::fs::OpenOptions::new().append(true).open(&path)?,
                ))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok((data, max_num, File::create_new(&path)?))
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl TableCore for Template<'static> {
    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn build(&self) -> Result<(), Error> {
        Ok(())
    }

    fn load(table: &ExcelTable, name: &str) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Self::load_template(table, name)
    }
}

pub struct Enums<'a> {
    sheets: Vec<(String, Sheet)>,
    mapping: Vec<(&'a str, HashMap<&'a str, &'a str>)>,
}

impl<'a> Enums<'a> {
    pub fn new() -> Self {
        Self { sheets: vec![], mapping: vec![] }
    }

    pub fn load_enum(&mut self, table: &ExcelTable, name: &str) -> Result<(), Error> {
        let sheet = Self::inner_load_sheet(table)?;
        self.sheets.push((name.into(), sheet));
        Ok(())
    }

    fn inner_load_sheet(table: &ExcelTable) -> Result<Sheet, Error> {
        let row = table.height();
        let col = table.width();

        // build row data
        let data = unsafe {
            let mut raw = Box::<[RowData]>::new_uninit_slice(row);
            for r in 0..row {
                let mut row_data = Box::<[String]>::new_uninit_slice(col);
                for c in 0..col {
                    row_data[c]
                        .as_mut_ptr()
                        .write(table.cell_content(c, r).unwrap_or("").trim().into());
                }
                raw[r].as_mut_ptr().write(RowData(row_data.assume_init()));
            }
            raw.assume_init()
        };

        Ok(Sheet {
            col,
            row,
            data,
        })
    }
}