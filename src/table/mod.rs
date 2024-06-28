use crate::{
    config::{CFG, CONFIG_COLLECTION_PATH},
    error::Error,
    lex::states::nodes::value_type,
    types::Value,
    THREADS,
};
use ansi_term::Colour::Red;
use dashmap::DashMap;
use global_config::GlobalConfig;
use language::Languages;
use std::{collections::HashMap, io::Write, sync::Arc};
use template::{Enums, Template};
use xlsx_read::excel_table::ExcelTable;

mod global_config;
mod language;
mod template;

pub enum TableEntity {
    Invalid,
    Template(
        String,
        Option<ExcelTable>,
        Vec<(String, ExcelTable)>,
        Vec<(String, ExcelTable)>,
    ), // (name, template, enums, extras)
    GlobalConfig(String, Option<ExcelTable>),
    Language(Vec<(String, ExcelTable)>),
}

unsafe impl Send for TableEntity {}
unsafe impl Sync for TableEntity {}

impl TableEntity {
    pub fn view<'a>(&'a self, ctx: Arc<BuildContext>) -> Result<Table<'a>, Error> {
        Table::load(self, ctx)
    }

    pub fn new_template(name: &str) -> Self {
        TableEntity::Template(name.into(), None, Vec::new(), Vec::new())
    }

    pub fn new_global(name: &str) -> Self {
        TableEntity::GlobalConfig(name.into(), None)
    }

    pub fn new_language(first: (String, ExcelTable)) -> Self {
        let mut data = Vec::new();
        data.push(first);
        TableEntity::Language(data)
    }

    #[inline]
    fn is_language(&self) -> bool {
        match self {
            TableEntity::Language(_) => true,
            _ => false,
        }
    }

    #[inline]
    fn is_valid(&self) -> bool {
        match self {
            TableEntity::Invalid => false,
            _ => true,
        }
    }

    #[inline]
    fn name(&self) -> &str {
        match self {
            TableEntity::Template(v, _, _, _) => &v,
            TableEntity::GlobalConfig(v, _) => &v,
            _ => "",
        }
    }
}

pub struct Generator<'a> {
    pub entities: Vec<TableEntity>,
    pub loption: &'a str,
}

impl Generator<'_> {
    pub fn build(self) -> Result<(), Error> {
        // generate ConfigCollection.cs
        let mut file = std::fs::File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(unsafe { CONFIG_COLLECTION_PATH })?;
        file.write_fmt(format_args!("{}", CFG.file_banner))?;
        file.write(
            r##"
using Config.Common;
using System.Collections.Generic;

namespace Config
{
    /// <summary>
    /// 所有配置数据类的集合
    /// </summary>
    public static class ConfigCollection
    {
        /// <summary>
        /// 所有配置数据类的集合
        /// </summary>
        public static readonly IConfigData[] Items = new IConfigData[]
        {"##
            .as_bytes(),
        )?;

        // TODO: 临时代码
        file.write_fmt(format_args!("\n\t\t\tLocalSurnames.Instance,"))?;
        file.write_fmt(format_args!("\n\t\t\tLocalNames.Instance,"))?;
        file.write_fmt(format_args!("\n\t\t\tLocalZangNames.Instance,"))?;
        file.write_fmt(format_args!("\n\t\t\tLocalTownNames.Instance,"))?;
        file.write_fmt(format_args!("\n\t\t\tLocalMonasticTitles.Instance,"))?;

        for name in self
            .entities
            .iter()
            .filter(|v| v.is_valid() && !v.is_language())
            .map(|v| v.name())
        {
            file.write_fmt(format_args!("\n\t\t\t{}.Instance,", name))?;
        }

        file.write("\n\t\t".as_bytes())?;
        file.write(r##"};

        /// <summary>
        /// 配置数据名称表
        /// </summary>
        public static readonly Dictionary<string, IConfigData> NameMap = new Dictionary<string, IConfigData>()
        {"##.as_bytes())?;

        // TODO: 临时代码
        file.write_fmt(format_args!(
            "\n\t\t\t{{\"{}\", {}.Instance}},",
            "LocalSurnames", "LocalSurnames"
        ))?;
        file.write_fmt(format_args!(
            "\n\t\t\t{{\"{}\", {}.Instance}},",
            "LocalNames", "LocalNames"
        ))?;
        file.write_fmt(format_args!(
            "\n\t\t\t{{\"{}\", {}.Instance}},",
            "LocalZangNames", "LocalZangNames"
        ))?;
        file.write_fmt(format_args!(
            "\n\t\t\t{{\"{}\", {}.Instance}},",
            "LocalTownNames", "LocalTownNames"
        ))?;
        file.write_fmt(format_args!(
            "\n\t\t\t{{\"{}\", {}.Instance}},",
            "LocalMonasticTitles", "LocalMonasticTitles"
        ))?;

        for name in self
            .entities
            .iter()
            .filter(|v| v.is_valid() && !v.is_language())
            .map(|v| v.name())
        {
            file.write_fmt(format_args!("\n\t\t\t{{\"{}\", {}.Instance}},", name, name))?;
        }
        file.write("\n\t\t".as_bytes())?;
        file.write(
            r##"};
    }
}"##
            .as_bytes(),
        )?;
        file.flush()?;

        // loading tables
        let ctx = std::sync::Arc::new(BuildContext {
            loption: self.loption,
            ..Default::default()
        });
        let mut views = vec![];
        THREADS.join(
            || println!("Getting views of tables..."),
            || {
                self.entities.iter().for_each(|v| {
                    let ctx = ctx.clone();
                    THREADS.install(|| views.push(v.view(ctx)));
                });
            },
        );

        // generate
        THREADS.join(
            || println!("Building codes..."),
            || {
                views.into_iter().for_each(|v| {
                    THREADS.install(|| match v {
                        Ok(mut view) => match view.build(ctx.as_ref()) {
                            Err(e) => {
                                eprintln!("{}", Red.bold().paint(format!("Build failed: {}", e)));
                            }
                            _ => {}
                        },
                        Err(e) => {
                            eprintln!("{}", Red.bold().paint(format!("Invalid tableview: {}", e)))
                        }
                    });
                })
            },
        );
        Ok(())
    }
}

#[derive(Default)]
pub struct BuildContext<'a> {
    pub(crate) refs: DashMap<String, (HashMap<String, i32>, i32)>,
    pub(crate) loption: &'a str,
}

#[allow(dead_code)]
pub trait TableCore<'a> {
    fn name(&self) -> &str;
    fn build<'b: 'a>(&mut self, ctx: &'b BuildContext) -> Result<(), Error>;
    fn load<'b: 'a>(
        _: &'b ExcelTable,
        _: &'b str,
        _: &'b [(String, ExcelTable)],
        _: Arc<BuildContext>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        unimplemented!()
    }
    fn load_language<'b: 'a>(
        _: &'b [(String, ExcelTable)],
        _: &'b str,
        _: Arc<BuildContext>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

pub struct Table<'a> {
    core: Option<Box<dyn TableCore<'a> + 'a>>,
}

unsafe impl Send for Table<'_> {}

impl<'a> Table<'a> {
    pub fn load<'b: 'a>(
        table: &'b TableEntity,
        ctx: std::sync::Arc<BuildContext>,
    ) -> Result<Self, Error> {
        #[allow(unused_assignments)]
        let mut core: Option<Box<dyn TableCore>> = None;

        match table {
            TableEntity::Invalid => return Err("Invalid TableEntity".into()),
            TableEntity::Template(name, template, menums, extras) => {
                let mut template = Template::load(
                    unsafe { template.as_ref().unwrap_unchecked() },
                    &name,
                    extras.as_slice(),
                    ctx.clone(),
                )?;

                if !menums.is_empty() {
                    let mut enums = Enums::new(name.as_str());
                    for (name, sheet) in menums.iter() {
                        enums.load_enum(sheet, name.as_str())?;
                    }
                    enums.establish();
                    template.enums = Some(enums);
                }
                core = Some(Box::new(template) as _);
            }
            TableEntity::GlobalConfig(name, global) => {
                core = Some(Box::new(GlobalConfig::load(
                    unsafe { global.as_ref().unwrap_unchecked() },
                    &name,
                    &[],
                    ctx.clone(),
                )?));
            }
            TableEntity::Language(langs) => {
                core = Some(Box::new(Languages::load_language(
                    langs.as_slice(),
                    "",
                    ctx.clone(),
                )?));
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

    pub fn build<'b: 'a>(&mut self, ctx: &'b BuildContext) -> Result<(), Error> {
        let Some(core) = self.core.as_mut() else {
            return Err("the core of Table is None".into());
        };
        core.build(ctx)?;
        Ok(())
    }
}

pub struct Sheet<'a> {
    col: usize,
    row: usize,
    data: Box<[VectorView<&'a str>]>,
}

#[allow(dead_code)]
impl<'a> Sheet<'a> {
    pub fn load<'b: 'a>(table: &'b ExcelTable) -> Result<Self, Error> {
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
                        .write(table.cell_content(c, r).unwrap_or(""));
                }
                raw[r]
                    .as_mut_ptr()
                    .write(VectorView(row_data.assume_init()));
            }
            raw.assume_init()
        };

        Ok(Sheet { col, row, data })
    }

    pub fn ty(&self, col: usize, row: usize) -> Result<Box<value_type>, Error> {
        if col < self.col && row < self.row {
            crate::parser::parse_type(self.data[row - CFG.row_of_start].value(col)?, 0, 0)
        } else {
            Err("Index was out of range".into())
        }
    }

    #[deprecated]
    pub fn value(
        &self,
        col: usize,
        row: usize,
        ty: &Box<value_type>,
        ls_map: Option<&HashMap<String, i32>>,
        ls_emptys: Option<&Vec<i32>>,
    ) -> Result<Box<dyn Value>, Error> {
        if col < self.col && row < self.row {
            crate::parser::parse_assign_with_type(
                ty,
                self.data[row - CFG.row_of_start].value(col)?,
                ls_map,
                ls_emptys,
            )
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

    pub fn row_iter(&self) -> SheetIterByRow<'_> {
        SheetIterByRow {
            view: &self.data,
            r: 0,
        }
    }

    pub fn cell(&self, col: usize, row: usize, trim: bool) -> Result<&str, Error> {
        if col < self.col && row < self.row {
            if trim {
                Ok(self.data[row].value(col)?.trim())
            } else {
                Ok(*self.data[row].value(col)?)
            }
        } else {
            Err("Index was out of range".into())
        }
    }
}

pub struct SheetFullIter<'a> {
    view: &'a [VectorView<&'a str>],
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

pub struct SheetIterByRow<'a> {
    view: &'a [VectorView<&'a str>],
    r: usize,
}

impl<'a> Iterator for SheetIterByRow<'a> {
    type Item = &'a VectorView<&'a str>;

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

/* #[repr(transparent)]
pub struct RowData<'a>(Box<[&'a str]>);

impl RowData<'_> {
    pub fn value(&self, col: usize) -> Result<&str, Error> {
        if col < self.0.len() {
            Ok(self.0[col])
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
    view: &'a [&'a str],
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
} */

#[repr(transparent)]
pub struct VectorView<T>(Box<[T]>);

impl<T> VectorView<T> {
    pub fn iter(&self) -> std::slice::Iter<T> {
        self.0.iter()
    }

    pub fn value(&self, idx: usize) -> Result<&T, Error> {
        if idx < self.0.len() {
            Ok(&self.0[idx])
        } else {
            Err("Exceeded the range of the row data index".into())
        }
    }
}
