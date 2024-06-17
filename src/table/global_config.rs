use xlsx_read::excel_table::ExcelTable;

use crate::{config::CFG, error::Error, util};

use super::{RowData, Sheet, TableCore};

pub struct GlobalConfig {
    name: String,
    main: Sheet,
}

impl TableCore for GlobalConfig {
    fn name(&self) -> &str {
        &self.name
    }

    fn build(&self, file: &mut dyn std::io::Write, _: bool) -> Result<(), Error> {
        writeln!(file, "{}", CFG.file_banner)?;
        writeln!(file, "using Config;")?;
        writeln!(file, "using System.Collections.Generic;")?;
        writeln!(file, "using Config.Common;")?;
        writeln!(file, "")?;
        writeln!(file, "#pragma warning disable 1591")?;
        writeln!(file, "")?;
        writeln!(file, "[System.Serializable]")?;
        writeln!(file, "public class {} : IConfigData", self.name)?;
        writeln!(file, "{{")?;
        writeln!(file, "    public int GetItemId(string refName) => throw new System.NotImplementedException();")?;
        writeln!(file, "    public int AddExtraItem(string identifier, string refName, object configItem) => throw new System.NotImplementedException();")?;
        writeln!(
            file,
            "    public static {} Instance = new {}();",
            self.name, self.name
        )?;
        writeln!(file, "")?;

        for row in self.main.iter() {
            let cols: Vec<&str> = row.iter().collect::<Vec<_>>();
            unsafe {
                let ident = cols.get_unchecked(0);
                let ty = cols.get_unchecked(1);
                let val = cols.get_unchecked(2);
                let desc = cols.get_unchecked(3);
                let modify = cols.get_unchecked(4);
                let value_ty = crate::parser::parse_type(*ty, 0, 0)?;
                let value = crate::parser::parse_assign_with_type(&value_ty, val)?;

                writeln!(file, "    /// <summary>")?;
                writeln!(file, "    /// {}", desc)?;
                writeln!(file, "    /// </summary>")?;

                if *modify == "0" {
                    write!(file, "    public {} {} = ", ty, ident)?;
                    value.code(file)?;
                    writeln!(file, ";")?;
                } else if value.ty_info().is_array_or_list() {
                    write!(file, "    public static readonly {} {} = ", ty, ident)?;
                    value.code(file)?;
                    writeln!(file, ";")?;
                } else {
                    write!(file, "    public const {} {} = ", ty, ident)?;
                    value.code(file)?;
                    writeln!(file, ";")?;
                }
            }
        }

        Ok(())
    }

    fn load(table: &ExcelTable, name: &str) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let row = table.height();
        let col = table.width();

        let data = unsafe {
            let mut raw = Box::<[RowData]>::new_uninit_slice(row - 1);
            for r in 1..row {
                let mut row_data = Box::<[String]>::new_uninit_slice(col);
                for c in 0..col {
                    row_data[c].as_mut_ptr().write(
                        table
                            .cell_content(c, r)
                            .ok_or::<Error>(
                                format!(
                                    "Load table cell failed: table name = {}, in cell ({}, {})",
                                    name,
                                    r + 1,
                                    util::conv_col_idx(c + 1)
                                )
                                .into(),
                            )?
                            .trim()
                            .into(),
                    );
                }
                raw[r].as_mut_ptr().write(RowData(row_data.assume_init()));
            }
            raw.assume_init()
        };

        Ok(Self {
            name: name.into(),
            main: Sheet { col, row, data },
        })
    }
}
