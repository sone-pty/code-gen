use std::sync::Arc;

use super::{BuildContext, Sheet, TableCore, VectorView};
use crate::{
    config::{CFG, OUTPUT_SCRIPT_CODE_DIR, OUTPUT_SERVER_SCRIPT_CODE_DIR},
    error::Error,
    types::Value,
    util::{self, conv_col_idx},
};
use xlsx_read::excel_table::ExcelTable;

pub struct GlobalConfig<'a> {
    name: &'a str,
    main: Sheet<'a>,
}

impl<'a> GlobalConfig<'a> {
    fn inner_build(
        &self,
        file: &mut dyn std::io::Write,
        values: &[Box<dyn Value>],
    ) -> Result<(), Error> {
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

        let mut vals = Vec::with_capacity(self.main.row);
        for (idx, row) in self.main.row_iter().skip(1).enumerate() {
            let cols: Vec<&str> = row.iter().map(|v| *v).collect::<Vec<_>>();
            unsafe {
                let ident = cols.get_unchecked(0);
                let ty = cols.get_unchecked(1);
                let desc = cols.get_unchecked(3);
                let modify = cols.get_unchecked(4);
                let value = values.get_unchecked(idx);

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
                vals.push((ident.to_string(), value, *modify == "0"));
            }
        }

        writeln!(file, "")?;
        writeln!(file, "    public void Init()")?;
        writeln!(file, "    {{")?;
        for (ident, value, modify) in vals.iter() {
            if *modify {
                write!(file, "        {} = ", ident)?;
                value.code(file)?;
                writeln!(file, ";")?;
            }
        }
        writeln!(file, "    }}")?;
        write!(file, "}}")?;
        Ok(())
    }
}

impl<'a> TableCore<'a> for GlobalConfig<'a> {
    fn name(&self) -> &str {
        &self.name
    }

    fn build<'b: 'a>(&mut self, _: &'b BuildContext) -> Result<(), Error> {
        let mut client_stream = std::fs::File::create(format!(
            "{}/{}.{}",
            unsafe { OUTPUT_SCRIPT_CODE_DIR },
            self.name,
            CFG.dest_code_suffix
        ))?;
        let mut server_stream = std::fs::File::create(format!(
            "{}/{}.{}",
            unsafe { OUTPUT_SERVER_SCRIPT_CODE_DIR },
            self.name,
            CFG.dest_code_suffix
        ))?;
        let values = {
            let mut values = Vec::with_capacity(self.main.row - 1);
            for (idx, row) in self.main.row_iter().skip(1).enumerate() {
                let ty = row.value(1)?;
                let val = row.value(2)?;
                let value_ty = crate::parser::parse_type(*ty, 0, 0)?;
                let tyinfo = crate::parser::get_value_type(&value_ty)?;

                if tyinfo.contains_str_type() {
                    let tval = crate::parser::transfer_str_value(val, &tyinfo)?;
                    let value =
                        match crate::parser::parse_assign_with_type(&value_ty, &tval, None, None) {
                            Ok(e) => e,
                            Err(e) => {
                                return Err(format!(
                                    "In table {}, the Cell.({}, {}) parse failed: {}",
                                    self.name,
                                    idx + 2,
                                    conv_col_idx(3),
                                    e
                                )
                                .into())
                            }
                        };
                    values.push(value);
                } else {
                    let value =
                        match crate::parser::parse_assign_with_type(&value_ty, val, None, None) {
                            Ok(e) => e,
                            Err(e) => {
                                return Err(format!(
                                    "In table {}, the Cell.({}, {}) parse failed: {}",
                                    self.name,
                                    idx + 2,
                                    conv_col_idx(3),
                                    e
                                )
                                .into())
                            }
                        };
                    values.push(value);
                }
            }
            values
        };
        self.inner_build(&mut client_stream, values.as_slice())?;
        self.inner_build(&mut server_stream, values.as_slice())?;
        Ok(())
    }

    fn load<'b: 'a>(
        table: &'b ExcelTable,
        name: &'b str,
        _: &'b [(String, ExcelTable)],
        _: Arc<BuildContext>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let row = table.height();
        let col = table.width();

        let data = unsafe {
            let mut raw = Box::<[VectorView<&str>]>::new_uninit_slice(row);
            for r in 0..row {
                let mut row_data = Box::<[&str]>::new_uninit_slice(col);
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
                raw[r]
                    .as_mut_ptr()
                    .write(VectorView(row_data.assume_init()));
            }
            raw.assume_init()
        };

        Ok(Self {
            name: name.into(),
            main: Sheet { col, row, data },
        })
    }
}
