use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, Write},
    sync::Arc,
};

use ansi_term::Colour::Red;
use xlsx_read::excel_table::ExcelTable;

use crate::{
    config::{CFG, REF_TEXT_DIR},
    error::Error,
    util,
};

use super::{BuildContext, RowData, Sheet, Table, TableCore};

pub struct Template<'a> {
    name: &'a str,
    pub(crate) enums: Option<Enums<'a>>,
    main: Sheet<'a>,
    fk_cols: Vec<usize>,
}

impl<'a> Template<'a> {
    fn load_template<'b: 'a>(
        table: &'b ExcelTable,
        name: &'b str,
        ctx: &BuildContext,
    ) -> Result<Self, Error> {
        let row = Table::get_sheet_height(table)?;
        let col = table.width();
        let mut table_refs_set = HashSet::new();
        
        // build row data
        let data = unsafe {
            let mut raw = Box::<[RowData]>::new_uninit_slice(row);
            for r in 0..row {
                let mut row_data = Box::<[&str]>::new_uninit_slice(col);
                for c in 0..col {
                    row_data[c]
                        .as_mut_ptr()
                        .write(table.cell_content(c, r).unwrap_or("").trim());
                }
                raw[r].as_mut_ptr().write(RowData(row_data.assume_init()));
            }
            raw.assume_init()
        };

        let (mut refs, mut max_ref_num, mut ref_file) = Self::load_refs(name)?;
        let init = max_ref_num == CFG.ref_start_num - 1;

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
        // flush
        ref_file.flush()?;

        ctx.refs.insert(name.into(), (refs, max_ref_num));
        Ok(Self {
            main: Sheet { col, row, data },
            enums: None,
            name: name.into(),
            fk_cols: vec![],
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

    fn load_fk_values<'c, 'b: 'c>(&mut self, ctx: &'b BuildContext) -> Result<FKValue<'c>, Error> {
        let refs = ctx.refs.get(self.name).ok_or::<Error>("Can't find refdata".into())?;
        for c in 0..self.main.col {
            let pattern = self.main.cell(c, CFG.row_of_fk)?;
            if pattern.starts_with('*') {
                self.fk_cols.push(c);
            }
        }
        FKValue::load(ctx)
    }
}

impl<'a> TableCore<'a> for Template<'a> {
    fn name(&self) -> &str {
        &self.name
    }

    fn build<'b: 'a>(&mut self, ctx: &'b BuildContext) -> Result<(), Error> {
        let fks = self.load_fk_values(&ctx)?;
        Ok(())
    }

    fn load<'b: 'a>(
        table: &'b ExcelTable,
        name: &'b str,
        ctx: Arc<BuildContext>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Self::load_template(table, name, ctx.as_ref())
    }
}

pub struct Enums<'a> {
    sheets: Vec<(&'a str, Sheet<'a>)>,
    mapping: Vec<(&'a str, HashMap<&'a str, &'a str>)>,
}

impl<'a> Enums<'a> {
    pub fn new() -> Self {
        Self {
            sheets: vec![],
            mapping: vec![],
        }
    }

    pub fn load_enum<'b: 'a>(&mut self, table: &'b ExcelTable, name: &'b str) -> Result<(), Error> {
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
                let mut row_data = Box::<[&str]>::new_uninit_slice(col);
                for c in 0..col {
                    row_data[c]
                        .as_mut_ptr()
                        .write(table.cell_content(c, r).unwrap_or("").trim().into());
                }
                raw[r].as_mut_ptr().write(RowData(row_data.assume_init()));
            }
            raw.assume_init()
        };

        Ok(Sheet { col, row, data })
    }
}

#[derive(Default)]
struct FKValue<'a> {
    newvals: HashMap<usize, Vec<&'a str>>,
}

impl<'a> FKValue<'a> {
    fn load<'b: 'a>(ctx: &'b BuildContext) -> Result<Self, Error> {
        Ok(FKValue::default())
    }
}