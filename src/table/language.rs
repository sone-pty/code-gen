use std::{fs::File, io::Write, sync::Arc};

use xlsx_read::excel_table::ExcelTable;

use crate::{
    config::{CFG, LANG_OUTPUT_DIR, OUTPUT_SCRIPT_CODE_DIR},
    error::Error,
};

use super::{BuildContext, Sheet, TableCore};

pub struct Languages<'a> {
    langs: Vec<(&'a str, Sheet<'a>)>,
}

impl<'a> TableCore<'a> for Languages<'a> {
    fn name(&self) -> &str {
        "LanguageKey"
    }

    fn build<'b: 'a>(&mut self, ctx: &'b BuildContext) -> Result<(), Error> {
        let path = format!(
            "{}/{}.{}",
            unsafe { LANG_OUTPUT_DIR },
            CFG.language_file_name,
            CFG.language_file_suffix
        );
        let mut langfile = File::create(path)?;
        let mut file = File::create(format!(
            "{}/{}.{}",
            unsafe { OUTPUT_SCRIPT_CODE_DIR },
            self.name(),
            CFG.dest_code_suffix
        ))?;
        let col = match ctx.loption {
            "CN" => 1,
            "CNH" => 2,
            "EN" => 3,
            "JP" => 4,
            _ => unreachable!(),
        };

        file.write_fmt(format_args!("{}{}", CFG.file_banner, CFG.line_end_flag))?;
        writeln!(file, "using System.Collections.Generic;")?;
        writeln!(file, "// ReSharper disable InconsistentNaming")?;
        writeln!(file, "// ReSharper disable IdentifierTypo")?;
        writeln!(file, "// ReSharper disable StringLiteralTypo")?;
        writeln!(file, "public class LanguageKey")?;
        writeln!(file, "{{")?;
        writeln!(file, "#region const keys")?;

        let mut count = 0;
        for (_, table) in self.langs.iter() {
            for row in 2..table.row {
                let v = table.cell(0, row, false)?;
                if !v.trim().is_empty() {
                    writeln!(
                        file,
                        "    public const ushort {} = {};",
                        table.cell(0, row, false)?,
                        count
                    )?;
                    count += 1;
                }
                let v = table.cell(col, row, false)?;
                if !v.trim().is_empty() {
                    writeln!(langfile, "{}", table.cell(col, row, false)?)?;
                }
            }
        }

        writeln!(file, "    public const ushort Invalid = ushort.MaxValue;")?;
        writeln!(file, "#endregion")?;
        writeln!(file, "")?;
        writeln!(
            file,
            "    public static ushort LanguageKeyToId(string languageKey)"
        )?;
        writeln!(file, "    {{")?;
        writeln!(
            file,
            "        if (_filedIdMap.TryGetValue(languageKey, out ushort id))"
        )?;
        writeln!(file, "            return id;")?;
        writeln!(file, "        return Invalid;")?;
        writeln!(file, "    }}")?;
        writeln!(file, "")?;

        writeln!(file, "    private static readonly Dictionary<string,ushort> _filedIdMap = new Dictionary<string,ushort>()")?;
        writeln!(file, "    {{")?;

        for (_, table) in self.langs.iter() {
            for row in 2..table.row {
                let v = table.cell(0, row, false)?;
                if !v.trim().is_empty() {
                    writeln!(file, "        {{\"{}\", {}}},", v, v)?;
                }
            }
        }
        writeln!(file, "    }};")?;
        writeln!(file, "}}")?;
        
        file.flush()?;
        langfile.flush()?;
        Ok(())
    }

    fn load_language<'b: 'a>(
        langs: &'b [(String, ExcelTable)],
        _: &'b str,
        _: Arc<BuildContext>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut data = Vec::new();
        for (name, sheet) in langs {
            data.push((name.as_str(), Sheet::load(sheet)?));
        }
        Ok(Self { langs: data })
    }
}
