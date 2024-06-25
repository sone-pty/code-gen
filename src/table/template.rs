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
    config::{
        CFG, ENUM_FLAGS_FILTER, LANG_OUTPUT_DIR, OUTPUT_ENUM_CODE_DIR, OUTPUT_SERVER_ENUM_CODE_DIR, REF_TEXT_DIR
    },
    error::Error,
    types::{TypeInfo, Value},
    util::{self, conv_col_idx},
};

use super::{BuildContext, Sheet, Table, TableCore, VectorView};

pub struct Template<'a> {
    name: &'a str,
    pub(crate) enums: Option<Enums<'a>>,
    main: Sheet<'a>,
    fk_cols: Vec<usize>,
    extras: Vec<(&'a str, &'a str)>,
}

impl<'a> Template<'a> {
    fn load_template<'b: 'a>(
        table: &'b ExcelTable,
        name: &'b str,
        extras: &'b [(String, ExcelTable)],
        ctx: &BuildContext,
    ) -> Result<Self, Error> {
        let row = Table::get_sheet_height(table)?;
        let col = table.width();
        let mut table_refs_set = HashSet::new();
        let mut extra_sheets = vec![];

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

        // extras
        for (_, sheet) in extras.iter() {
            for r in 2..sheet.height() {
                match (sheet.cell_content(0, r), sheet.cell_content(1, r)) {
                    (Some(v1), Some(v2)) => {
                        extra_sheets.push((v1, v2));
                    }
                    _ => {}
                }
            }
        }

        ctx.refs.insert(name.into(), (refs, max_ref_num));
        Ok(Self {
            main: Sheet { col, row, data },
            enums: None,
            name: name.into(),
            fk_cols: vec![],
            extras: extra_sheets,
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

    fn build_fk_values<'c, 'b: 'c>(&mut self, ctx: &'b BuildContext) -> Result<FKValue<'c>, Error> {
        for c in 0..self.main.col {
            let pattern = self.main.cell(c, CFG.row_of_fk)?;
            if pattern.starts_with('*') {
                self.fk_cols.push(c);
            }
        }
        FKValue::load(ctx, self.fk_cols.as_slice(), self.main.data.as_ref())
    }

    fn build_lstring_values(
        &self,
    ) -> Result<(HashMap<String, i32>, HashMap<(usize, usize), Vec<i32>>), Error> {
        let mut seed = 0i32;
        let mut ls_map = HashMap::new();
        let mut emptys: HashMap<(usize, usize), Vec<i32>> = HashMap::new();
        let mut path = std::path::Path::new(unsafe { LANG_OUTPUT_DIR }).to_path_buf();
        path.push(format!("{}_language", self.name));
        path.set_extension("txt");
        let mut file = File::create(path.as_path())?;

        for c in (0..self.main.col).filter(|v| {
            self.main
                .cell(*v, CFG.row_of_type)
                .is_ok_and(|v| v.contains("Lstring") || v.contains("LString"))
        }) {
            let default = self.main.cell(c, CFG.row_of_default)?;
            let ty = self.main.cell(c, CFG.row_of_type)?;

            'row: for r in CFG.row_of_start..self.main.row {
                let val = {
                    let v = self.main.cell(c, r)?;
                    if v.is_empty() {
                        default
                    } else {
                        v
                    }
                };
                let pos = (c, r);
                let trivial = { ty == "LString" || ty == "Lstring" };

                if !trivial {
                    let fval = val.chars().filter(|c| *c != ' ').collect::<String>();

                    if fval.is_empty() || val == "{}" {
                        continue 'row;
                    }

                    if !fval.starts_with('{') || !fval.ends_with('}') {
                        return Err("Invalid format".into());
                    }

                    let val = &val[1..val.len() - 1];
                    let raw_elements: Vec<&str> = val.split(',').collect();

                    for v in raw_elements {
                        if v.is_empty() {
                            match emptys.entry(pos) {
                                std::collections::hash_map::Entry::Occupied(mut e) => {
                                    e.get_mut().push(seed);
                                }
                                std::collections::hash_map::Entry::Vacant(e) => {
                                    let mut data = Vec::new();
                                    data.push(seed);
                                    e.insert(data);
                                }
                            }
                            seed += 1;
                            writeln!(file, "")?;
                        } else {
                            match ls_map.entry(v.into()) {
                                std::collections::hash_map::Entry::Vacant(e) => {
                                    writeln!(file, "{}", v)?;
                                    e.insert(seed);
                                    seed += 1;
                                }
                                _ => {}
                            }
                        }
                    }
                } else {
                    if val.is_empty() {
                        match emptys.entry(pos) {
                            std::collections::hash_map::Entry::Occupied(mut e) => {
                                e.get_mut().push(seed);
                            }
                            std::collections::hash_map::Entry::Vacant(e) => {
                                let mut data = Vec::new();
                                data.push(seed);
                                e.insert(data);
                            }
                        }
                        seed += 1;
                        writeln!(file, "")?;
                    } else {
                        match ls_map.entry(val.into()) {
                            std::collections::hash_map::Entry::Vacant(e) => {
                                e.insert(seed);
                                seed += 1;
                                writeln!(file, "{}", val)?;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // extra language entrys
        writeln!(file, ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>")?;
        for (v1, v2) in self.extras.iter() {
            writeln!(file, "{}={}", v1, v2)?;
        }

        file.flush()?;
        Ok((ls_map, emptys))
    }

    fn inner_build_client(&self, ctx: &InnerBuildContext<'_>) -> Result<(), Error> {
        
        Ok(())
    }

    fn inner_build_server(&self, ctx: &InnerBuildContext<'_>) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a> TableCore<'a> for Template<'a> {
    fn name(&self) -> &str {
        &self.name
    }

    fn build<'b: 'a>(&mut self, ctx: &'b BuildContext) -> Result<(), Error> {
        // transfer fk values
        let fks = self.build_fk_values(&ctx)?;
        // transfer lstrings
        let (ls_map, emptys) = self.build_lstring_values()?;
        let mut defkey = CFG.cell_of_defkey.0;
        let mut skip_cols = Vec::new();
        let mut required = Vec::new();
        let mut nodefs = HashSet::new();
        let mut defaults = HashMap::new();
        let mut templates = Vec::with_capacity(self.main.row);
        let mut items = Vec::new();
        let mut enumflags: HashMap<_, Vec<&str>> = HashMap::new();
        let mut keytypes = None;

        // collect skip_cols and required fields and defkeys and enum flags
        for c in 0..self.main.col {
            let ident = self.main.cell(c, CFG.row_of_ident)?;

            if ident.starts_with('#') {
                skip_cols.push(c);
                if ident.contains("DefKey") {
                    defkey = c;
                    keytypes = Some(Vec::new());
                }
            } else if ident.is_empty() {
                skip_cols.push(c);
            } else {
                required.push(ident);
            }
        }

        // collect defkey
        if let Some(ref mut vec) = keytypes {
            for r in CFG.row_of_start..self.main.row {
                let v0 = self.main.cell(0, r)?;
                let v1 = self.main.cell(defkey, r)?;
                if !v0.is_empty() && !v1.is_empty() {
                    vec.push((v1, r - CFG.row_of_start, v0));
                }
            }
        }

        // template ids
        for r in (CFG.row_of_start..self.main.row)
            .map(|v| self.main.cell(0, v))
            .filter(|v| v.as_ref().is_ok_and(|v| !v.is_empty()))
        {
            templates.push(r?);
        }

        // parse values
        let mut values = Vec::new();
        for c in (1..self.main.col).filter(|v| !skip_cols.as_slice().contains(v)) {
            let mut rows = Vec::with_capacity(self.main.row - CFG.row_of_start + 1);
            let ident = self.main.cell(c, CFG.row_of_ident)?;
            let ty = self.main.cell(c, CFG.row_of_type)?;
            let ety = format!("enum {}.{}", self.name, ident);
            let value_ty = {
                if ty == "enum" {
                    crate::parser::parse_type(ety.as_str(), 0, 0)?
                } else {
                    crate::parser::parse_type(ty, 0, 0)?
                }
            };
            let tyinfo = crate::parser::get_value_type(&value_ty)?;
            let enum_flag = self.main.cell(c, CFG.row_of_enum)?;
            let default = self.main.cell(c, CFG.row_of_default)?;

            let get_value = |r: usize, idx: usize| -> Result<&str, Error> {
                let val = self.main.cell(c, r)?;
                let val = if val.is_empty() { default } else { val };
                if self.fk_cols.contains(&c) {
                    Ok(fks
                        .newvals
                        .get(&c)
                        .ok_or::<Error>(
                            format!(
                                "Can't find fk value when parsing Cell.({}, {}) in the table `{}`",
                                r + 1,
                                conv_col_idx(c + 1),
                                self.name
                            )
                            .into(),
                        )?
                        .value(idx)?)
                } else if ty == "enum" {
                    unsafe { self.enums.as_ref().unwrap_unchecked().get_value(ident, val) }
                } else {
                    Ok(val)
                }
            };

            if !enum_flag.is_empty() && !ENUM_FLAGS_FILTER.contains(enum_flag) && enum_flag.chars().all(|c| c.is_alphabetic()) {
                match enumflags.entry(enum_flag) {
                    std::collections::hash_map::Entry::Occupied(mut e) => {
                        e.get_mut().push(ident);
                    },
                    std::collections::hash_map::Entry::Vacant(e) => {
                        let mut data = Vec::with_capacity(10);
                        data.push(ident);
                        e.insert(data);
                    },
                }
            }

            // comments
            items.push((self.main.cell(c, CFG.row_of_comment)?, ident, ty));
            // defaults
            if default.is_empty() || default == "None" {
                nodefs.insert(ident);
            } else {
                match defaults.entry(ident) {
                    std::collections::hash_map::Entry::Vacant(e) => {
                        let val = get_value(CFG.row_of_default, 0)?;

                        if tyinfo.is_lstring_or_lstringarr() {
                            e.insert((tyinfo.clone(), None));
                        } else {
                            if tyinfo.contains_str_type() {
                                let tval = crate::parser::transfer_str_value(val, &tyinfo)?;
                                let value = match crate::parser::parse_assign_with_type(
                                    &value_ty,
                                    tval.as_str(),
                                    None,
                                    None,
                                ) {
                                    Ok(e) => e,
                                    Err(e) => {
                                        return Err(format!(
                                            "In table {}, the Cell.({}, {}) parse failed: {}",
                                            self.name,
                                            CFG.row_of_default + 1,
                                            conv_col_idx(c + 1),
                                            e,
                                        )
                                        .into())
                                    }
                                };
                                e.insert((tyinfo.clone(), Some(value)));
                            } else {
                                let value = match crate::parser::parse_assign_with_type(
                                    &value_ty, val, None, None,
                                ) {
                                    Ok(e) => e,
                                    Err(e) => {
                                        return Err(format!(
                                            "In table {}, the Cell.({}, {}) parse failed: {}",
                                            self.name,
                                            CFG.row_of_default + 1,
                                            conv_col_idx(c + 1),
                                            e,
                                        )
                                        .into())
                                    }
                                };
                                e.insert((tyinfo.clone(), Some(value)));
                            }
                        }
                    }
                    _ => {}
                }
            }

            // data rows
            for r in CFG.row_of_start..self.main.row {
                let pos = (c, r);
                let val = get_value(r, r - CFG.row_of_start + 1)?;

                if tyinfo.contains_str_type() {
                    let tval = crate::parser::transfer_str_value(val, &tyinfo)?;
                    let value = match crate::parser::parse_assign_with_type(
                        &value_ty,
                        &tval,
                        Some(&ls_map),
                        emptys.get(&pos),
                    ) {
                        Ok(e) => e,
                        Err(e) => {
                            return Err(format!(
                                "In table {}, the Cell.({}, {}) parse failed: {}",
                                self.name,
                                r + 1,
                                conv_col_idx(c + 1),
                                e,
                            )
                            .into())
                        }
                    };
                    rows.push(value);
                } else {
                    let value = match crate::parser::parse_assign_with_type(
                        &value_ty,
                        val,
                        Some(&ls_map),
                        emptys.get(&pos),
                    ) {
                        Ok(e) => e,
                        Err(e) => {
                            return Err(format!(
                                "In table {}, the Cell.({}, {}) parse failed: {}",
                                self.name,
                                r + 1,
                                conv_col_idx(c + 1),
                                e,
                            )
                            .into())
                        }
                    };
                    rows.push(value);
                }
            }
            values.push(rows);
        }

        // build
        let inner_ctx = InnerBuildContext {
            skip_cols,
            values,
            nodefs,
            defaults,
            templates,
            items,
            enumflags,
            keytypes,
        };
        self.inner_build_client(&inner_ctx)?;
        self.inner_build_server(&inner_ctx)?;
        Ok(())
    }

    fn load<'b: 'a>(
        table: &'b ExcelTable,
        name: &'b str,
        extras: &'b [(String, ExcelTable)],
        ctx: Arc<BuildContext>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Self::load_template(table, name, extras, ctx.as_ref())
    }
}

struct InnerBuildContext<'a> {
    skip_cols: Vec<usize>,
    values: Vec<Vec<Box<dyn Value>>>,
    nodefs: HashSet<&'a str>,
    defaults: HashMap<&'a str, (TypeInfo, Option<Box<dyn Value>>)>,
    templates: Vec<&'a str>,
    items: Vec<(&'a str, &'a str, &'a str)>,
    enumflags: HashMap<&'a str, Vec<&'a str>>,
    keytypes: Option<Vec<(&'a str, usize, &'a str)>>,
}

pub struct Enums<'a> {
    base: &'a str,
    mapping: Vec<(&'a str, HashMap<(*const u8, usize), (*const u8, usize)>)>,
}

impl<'a> Enums<'a> {
    pub fn new<'b: 'a>(base: &'b str) -> Self {
        Self {
            base,
            mapping: vec![],
        }
    }

    #[inline]
    pub fn establish(&mut self) {
        self.mapping.sort_by(|a, b| a.0.cmp(b.0));
    }

    pub fn get_value(&self, name: &str, key: &str) -> Result<&str, Error> {
        match self.mapping.binary_search_by(|v| v.0.cmp(name)) {
            Ok(idx) => {
                let mapping = unsafe { &self.mapping.get_unchecked(idx).1 };
                unsafe {
                    let meta = mapping
                        .get(&(key.as_ptr(), key.len()))
                        .ok_or::<Error>("".into())?;
                    Ok(std::str::from_raw_parts(meta.0, meta.1))
                }
            }
            _ => Err("".into()),
        }
    }

    pub fn load_enum<'b: 'a>(&mut self, table: &'b ExcelTable, name: &'b str) -> Result<(), Error> {
        let sheet = Self::inner_load_sheet(table)?;
        self.save_to(
            &mut File::create(format!(
                "{}/E{}{}.cs",
                unsafe { OUTPUT_ENUM_CODE_DIR },
                self.base,
                name,
            ))?,
            &sheet,
            name,
        )?;
        self.save_to(
            &mut File::create(format!(
                "{}/E{}{}.cs",
                unsafe { OUTPUT_SERVER_ENUM_CODE_DIR },
                self.base,
                name,
            ))?,
            &sheet,
            name,
        )?;

        let mut esmap = HashMap::new();
        for r in 0..sheet.row {
            let ident = sheet.cell(CFG.col_of_enum_ident, r)?;
            let desc = sheet.cell(CFG.col_of_enum_desc, r)?;
            esmap.insert((desc.as_ptr(), desc.len()), (ident.as_ptr(), ident.len()));
        }
        self.mapping.push((name, esmap));
        Ok(())
    }

    fn save_to<W: std::io::Write + ?Sized>(
        &mut self,
        file: &mut W,
        sheet: &Sheet<'a>,
        name: &'a str,
    ) -> Result<(), Error> {
        file.write("#pragma warning disable 1591".as_bytes())?;
        file.write(CFG.line_end_flag.as_bytes())?;
        file.write(CFG.line_end_flag.as_bytes())?;
        file.write("/// <summary>".as_bytes())?;
        file.write(CFG.line_end_flag.as_bytes())?;
        file.write_fmt(format_args!(
            "/// {} -> {}{}",
            self.base, name, CFG.line_end_flag,
        ))?;
        file.write("/// </summary>".as_bytes())?;
        file.write(CFG.line_end_flag.as_bytes())?;
        file.write_fmt(format_args!(
            "public enum E{}{}{}",
            self.base, name, CFG.line_end_flag,
        ))?;
        file.write("{".as_bytes())?;
        file.write(CFG.line_end_flag.as_bytes())?;

        for r in 0..sheet.row {
            let ident = sheet.cell(CFG.col_of_enum_ident, r)?;
            let val = sheet.cell(CFG.col_of_enum_val, r)?;
            let desc = sheet.cell(CFG.col_of_enum_desc, r)?;

            file.write_fmt(format_args!("{}/// <summary>{}", '\t', CFG.line_end_flag))?;
            file.write_fmt(format_args!("{}/// {}{}", '\t', desc, CFG.line_end_flag))?;
            file.write_fmt(format_args!("{}/// </summary>{}", '\t', CFG.line_end_flag))?;
            file.write_fmt(format_args!(
                "{}{} = {},{}",
                '\t', ident, val, CFG.line_end_flag
            ))?;
        }

        file.write_fmt(format_args!("{}Count{}", '\t', CFG.line_end_flag))?;
        file.write("}".as_bytes())?;
        file.flush()?;
        Ok(())
    }

    pub fn inner_load_sheet<'b: 'a>(table: &'b ExcelTable) -> Result<Sheet<'b>, Error> {
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
            let mut raw = Box::<[String]>::new_uninit_slice(data.len() - CFG.row_of_start + 1);
            let pattern = &data[CFG.row_of_fk].value(*c)?[1..];
            let default = data[CFG.row_of_default].value(*c)?;

            unsafe {
                raw[0]
                    .as_mut_ptr()
                    .write(Self::load_0(*&default, pattern, ctx)?);
            }

            for r in CFG.row_of_start..data.len() {
                let val = {
                    let val = data[r].value(*c)?;
                    if val.is_empty() {
                        *default
                    } else {
                        *val
                    }
                };

                unsafe {
                    raw[r - CFG.row_of_start + 1]
                        .as_mut_ptr()
                        .write(Self::load_0(val, pattern, ctx)?)
                };
            }
            ret.newvals
                .insert(*c, VectorView(unsafe { raw.assume_init() }));
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
        let patterns = util::split(pattern);
        let plen = patterns.len();
        let vals = util::split(value);
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
                    /* return Err(format!(
                        "Can't find ref about key `{}` in table {}",
                        v.1, fk_names[num]
                    )
                    .into()); */
                    output.push_str("-1");
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
        let patterns = util::split(pattern);
        let plen = patterns.len();
        let vals = util::split(value);
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
                    /* return Err(
                        format!("Can't find ref about key `{}` in table {}", v.1, pat).into(),
                    ); */
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
                /* return Err(
                    format!("Can't find ref about key `{}` in table {}", rev, refs.key()).into(),
                ); */
                dest.push_str("-1");
            }
        }
        Ok(())
    }
}
