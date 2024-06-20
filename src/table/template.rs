use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, Write},
    marker::PhantomData,
    sync::Arc,
};

use ansi_term::Colour::Red;
use xlsx_read::excel_table::ExcelTable;

use crate::{
    config::{CFG, REF_TEXT_DIR},
    error::Error,
    util,
};

use super::{BuildContext, Sheet, Table, TableCore, VectorView};

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
            let mut raw = Box::<[VectorView<&str>]>::new_uninit_slice(row);
            for r in 0..row {
                let mut row_data = Box::<[&str]>::new_uninit_slice(col);
                for c in 0..col {
                    row_data[c]
                        .as_mut_ptr()
                        .write(table.cell_content(c, r).unwrap_or("").trim());
                }
                raw[r]
                    .as_mut_ptr()
                    .write(VectorView(row_data.assume_init()));
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
            table_refs_set.insert(*data[r].value(0)?);
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
        for c in 0..self.main.col {
            let pattern = self.main.cell(c, CFG.row_of_fk)?;
            if pattern.starts_with('*') {
                self.fk_cols.push(c);
            }
        }
        FKValue::load(ctx, self.fk_cols.as_slice(), self.main.data.as_ref())
    }
}

impl<'a> TableCore<'a> for Template<'a> {
    fn name(&self) -> &str {
        &self.name
    }

    fn build<'b: 'a>(&mut self, ctx: &'b BuildContext) -> Result<(), Error> {
        // transfer fk values
        let fks = self.load_fk_values(&ctx)?;
        // transfer lstrings
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
            let mut raw = Box::<[VectorView<&str>]>::new_uninit_slice(row);
            for r in 0..row {
                let mut row_data = Box::<[&str]>::new_uninit_slice(col);
                for c in 0..col {
                    row_data[c]
                        .as_mut_ptr()
                        .write(table.cell_content(c, r).unwrap_or("").trim().into());
                }
                raw[r]
                    .as_mut_ptr()
                    .write(VectorView(row_data.assume_init()));
            }
            raw.assume_init()
        };

        Ok(Sheet { col, row, data })
    }
}

#[derive(Default)]
struct FKValue<'a> {
    newvals: HashMap<usize, VectorView<String>>,
    ph: PhantomData<&'a ()>,
}

impl<'a> FKValue<'a> {
    fn load<'b: 'a>(
        ctx: &'b BuildContext,
        cols: &[usize],
        data: &[VectorView<&str>],
    ) -> Result<Self, Error> {
        let mut ret = FKValue::default();
        for c in cols {
            let mut raw = Box::<[String]>::new_uninit_slice(data.len() - CFG.row_of_start);
            let pattern = data[CFG.row_of_fk].value(*c)?;
            let default = data[CFG.row_of_default].value(*c)?;
            for r in CFG.row_of_start..data.len() {
                let val = {
                    if pattern.is_empty() {
                        *default
                    } else {
                        *pattern
                    }
                };

                unsafe { raw[r - CFG.row_of_start].as_mut_ptr().write(Self::load_0(val, *&pattern, ctx)?) };
            }
            ret.newvals.insert(*c, VectorView(unsafe { raw.assume_init() }));
        }
        Ok(ret)
    }

    fn load_0(val: &str, pattern: &str, ctx: &BuildContext) -> Result<String, Error> {
        let rval = val.chars().filter(|c| *c != ' ').collect::<String>();
        let pat = pattern.chars().filter(|c| *c != ' ').collect::<String>();
        let mut ret = String::new();

        if pattern.chars().all(|c| c.is_alphanumeric())
            || pattern
                .chars()
                .filter(|c| *c != '{' && *c != '}')
                .all(|c| c.is_alphanumeric())
        {
            let mut ch_stack = util::Stack::<char>::new();
            let key = pattern
                .chars()
                .filter(|c| *c != '{' && *c != '}')
                .collect::<String>();

            let refs = ctx
                .refs
                .get(&key)
                .ok_or::<Error>(format!("Can't find refdata about `{}`", key).into())?;

            for v in rval.chars() {
                match v {
                    '{' => {
                        ret.push(v);
                    }
                    '}' | ',' | '，' => {
                        Self::replace(&mut ch_stack, &mut ret, &refs)?;
                        ret.push(v);
                    }
                    _ => {
                        ch_stack.push(v);
                    }
                }
            }

            if !ch_stack.is_empty() {
                Self::replace(&mut ch_stack, &mut ret, &refs)?;
            }
        } else if pattern.contains('?') || pattern.contains('#') {
            if rval != "{}" {
                ret.push('{');
            }
            Self::load_1(ctx, &pat, &rval, &mut ret)?;
            if rval != "{}" {
                ret.push('}');
            }
        } else {
            if rval != "{}" {
                ret.push('{');
            }
            Self::load_2(ctx, &pat, &rval, &mut ret)?;
            if rval != "{}" {
                ret.push('}');
            }
        }
        Ok(ret)
    }

    fn load_1(
        ctx: &BuildContext,
        pattern: &str,
        value: &str,
        output: &mut String,
    ) -> Result<(), Error> {
        let patterns = Self::split(pattern);
        let plen = patterns.len();
        let vals = Self::split(value);
        if plen == 0 {
            return Ok(());
        }
        let mut fk_names = Vec::new();

        for v in vals.iter().enumerate() {
            let pat = patterns[if v.0 < plen { v.0 } else { plen - 1 }];

            if pat.contains("{") || pat.contains("}") {
                output.push('{');
                Self::load_1(ctx, pat, v.1, output)?;
                output.push('}');
            } else if pat.is_empty() {
                output.push_str(v.1);
            } else if pat.starts_with("?") {
                output.push_str(v.1);
                // process num
                let num_str = &pat[1..];
                let mut num = 0;
                if !num_str.is_empty() {
                    num = num_str.parse()?;
                }
                fk_names.resize((num << 1) + 1, "");
                fk_names[num] = &v.1[1..v.1.len() - 1];
            } else if pat.starts_with("#") {
                // process num
                let num_str = &pat[1..];
                let mut num = 0;
                if !num_str.is_empty() {
                    num = num_str.parse()?;
                }

                let refs = ctx.refs.get(fk_names[num]).ok_or::<Error>(
                    format!("Can't find refdata about `{}`", fk_names[num]).into(),
                )?;

                if *v.1 == "None" {
                    output.push_str("-1");
                } else if let Some(replace) = refs.0.get(*v.1) {
                    std::fmt::Write::write_fmt(output, format_args!("{}", *replace))?;
                } else {
                    return Err(format!(
                        "Can't find ref about key `{}` in table {}",
                        v.1, fk_names[num]
                    )
                    .into());
                    //output.push_str("-1");
                }
            }

            if v.0 != vals.len() - 1 {
                output.push(',')
            };
        }
        Ok(())
    }

    fn load_2(
        ctx: &BuildContext,
        pattern: &str,
        value: &str,
        output: &mut String,
    ) -> Result<(), Error> {
        let patterns = Self::split(pattern);
        let plen = patterns.len();
        let vals = Self::split(value);
        if plen == 0 {
            return Ok(());
        }

        for v in vals.iter().enumerate() {
            let pat = patterns[if v.0 < plen { v.0 } else { plen - 1 }];

            if pat.contains("{") || pat.contains("}") {
                output.push('{');
                Self::load_2(ctx, pat, v.1, output)?;
                output.push('}');
            } else if pat.is_empty() {
                output.push_str(v.1);
            } else {
                let refs = ctx
                    .refs
                    .get(pat)
                    .ok_or::<Error>(format!("Can't find refdata about `{}`", pat).into())?;
                if *v.1 == "None" {
                    output.push_str("-1");
                } else if let Some(replace) = refs.0.get(*v.1) {
                    std::fmt::Write::write_fmt(output, format_args!("{}", *replace))?;
                } else {
                    return Err(
                        format!("Can't find ref about key `{}` in table {}", v.1, pat).into(),
                    );
                }
            }

            if v.0 != vals.len() - 1 {
                output.push(',')
            };
        }
        Ok(())
    }

    fn replace(
        st: &mut util::Stack<char>,
        dest: &mut String,
        refs: &dashmap::mapref::one::Ref<String, (HashMap<String, i32>, i32)>,
    ) -> Result<(), Error> {
        let mut s = String::with_capacity(10);
        while !st.is_empty() {
            if let Ok(r) = st.pop() {
                s.push(r)
            }
        }
        let rev: String = s.chars().rev().collect();
        if !rev.is_empty() {
            if let Some(v) = refs.0.get(&rev) {
                std::fmt::Write::write_fmt(dest, format_args!("{}", *v))?;
            } else {
                return Err(
                    format!("Can't find ref about key `{}` in table {}", rev, refs.key()).into(),
                );
                //dest.push_str("-1");
            }
        }
        Ok(())
    }

    fn split(pat: &str) -> Vec<&str> {
        let pat_trim = pat.trim();
        let mut ret = Vec::new();

        if pat_trim.starts_with("{") && pat_trim.ends_with("}") {
            let mut brackets = util::Stack::new();
            let mut begin = 1;
            let mut idx = 0;

            for v in pat_trim.chars() {
                match v {
                    '{' => {
                        if idx != 0 {
                            if brackets.is_empty() {
                                begin = idx;
                            }
                            brackets.push(v);
                        }
                    }
                    '}' => {
                        if idx == pat_trim.len() - 1 {
                            if begin < idx {
                                ret.push(&pat_trim[begin..idx]);
                            } else {
                                ret.push("");
                            }
                        } else {
                            let _ = brackets.pop();
                        }
                    }
                    ',' => {
                        if brackets.is_empty() {
                            ret.push(&pat_trim[begin..idx]);
                            begin = idx + 1;
                        }
                    }
                    _ => {}
                }
                idx += v.len_utf8();
            }
        }
        ret
    }
}
