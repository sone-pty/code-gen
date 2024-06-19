use crate::{
    config::{CFG, CONFIG_COLLECTION_PATH},
    error::Error,
    lex::states::nodes::value_type,
    types::Value,
    THREADS,
};
use ansi_term::Colour::Red;
use global_config::GlobalConfig;
use std::io::Write;
use template::{Enums, Template};
use xlsx_read::excel_table::ExcelTable;

mod global_config;
mod template;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum TableTy {
    Invalid,
    Template,
    GlobalConfig,
    Language,
}

#[derive(Default)]
pub struct TableEntity {
    pub template: Option<ExcelTable>,
    pub global: Option<ExcelTable>,
    pub enums: Vec<(String, ExcelTable)>,
    pub name: String,
}

unsafe impl Send for TableEntity {}

impl TableEntity {
    pub fn view(&self) -> Result<Table<'_>, Error> {
        Table::load(self)
    }

    pub fn ty(&self) -> TableTy {
        if self.template.is_some() {
            return TableTy::Template;
        }
        if self.global.is_some() {
            return TableTy::GlobalConfig;
        }
        TableTy::Invalid
    }
}

pub struct Generator {
    pub entities: Vec<TableEntity>,
}

impl Generator {
    pub fn build(self) -> Result<(), Error> {
        // generate ConfigCollection.cs
        let mut file = std::fs::File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(unsafe { CONFIG_COLLECTION_PATH })?;
        file.write_fmt(format_args!("{}{}", CFG.file_banner, CFG.line_end_flag))?;
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

        for name in self.entities.iter().map(|v| v.name.as_str()) {
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

        for name in self.entities.iter().map(|v| v.name.as_str()) {
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

        // generate the rest
        self.entities.into_iter().for_each(|v| {
            THREADS.spawn(move || match v.view() {
                Ok(view) => match view.build() {
                    Err(e) => {
                        eprintln!("{}", Red.bold().paint(format!("{}", e)));
                    }
                    _ => {}
                },
                Err(e) => eprintln!("{}", Red.bold().paint(format!("{}", e))),
            });
        });
        Ok(())
    }
}

#[allow(dead_code)]
pub trait TableCore<'a> {
    fn name(&self) -> &str;
    fn build(&self) -> Result<(), Error>;
    fn load<'b: 'a>(table: &'b ExcelTable, name: &'b str) -> Result<Self, Error> where Self: Sized;
}

pub struct Table<'a> {
    core: Option<Box<dyn TableCore<'a> + 'a>>,
}

unsafe impl Send for Table<'_> {}

impl<'a> Table<'a> {
    pub fn load<'b: 'a>(table: &'b TableEntity) -> Result<Self, Error> {
        let mut core: Option<Box<dyn TableCore>> = None;
        match table.ty() {
            TableTy::Template => {
                let mut template = Template::load(
                    unsafe { table.template.as_ref().unwrap_unchecked() },
                    &table.name,
                )?;
                if !table.enums.is_empty() {
                    let mut enums = Enums::new();
                    for (name, sheet) in table.enums.iter() {
                        enums.load_enum(sheet, name.as_str())?;
                    }
                    template.enums = Some(enums);
                }
                core = Some(Box::new(template) as _);
            }
            TableTy::GlobalConfig => {
                core = Some(Box::new(GlobalConfig::load(
                    unsafe { table.global.as_ref().unwrap_unchecked() },
                    &table.name,
                )?));
            }
            TableTy::Language => todo!(),
            TableTy::Invalid => {}
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

    pub fn build(&self) -> Result<(), Error> {
        let Some(ref core) = self.core else {
            return Err("the core of Table is None".into());
        };
        core.build()?;
        Ok(())
    }
}

pub struct Sheet<'a> {
    col: usize,
    row: usize,
    data: Box<[RowData<'a>]>,
}

#[allow(dead_code)]
impl Sheet<'_> {
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

    pub fn cell(&self, col: usize, row: usize) -> Result<&str, Error> {
        if col < self.col && row < self.row {
            self.data[row].value(col)
        } else {
            Err("Index was out of range".into())
        }
    }
}

pub struct SheetFullIter<'a> {
    view: &'a [RowData<'a>],
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
    view: &'a [RowData<'a>],
    r: usize,
}

impl<'a> Iterator for SheetIter<'a> {
    type Item = &'a RowData<'a>;

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
}
