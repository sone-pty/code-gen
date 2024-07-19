use super::TableCore;
use crate::table::Table;
use std::collections::HashMap;

pub struct FkTable<'a> {
    name: &'a str,
}

impl<'a> TableCore<'a> for FkTable<'a> {
    fn name(&self) -> &str {
        &self.name
    }

    fn build<'b: 'a>(&mut self, _: &'b super::BuildContext) -> Result<(), crate::error::Error> {
        Ok(())
    }

    fn load<'b: 'a>(
        table: &'b xlsx_read::excel_table::ExcelTable,
        name: &'b str,
        _: &'b [(String, super::ExcelTableWrapper)],
        ctx: std::sync::Arc<super::BuildContext>,
    ) -> Result<Self, crate::error::Error>
    where
        Self: Sized,
    {
        let row = Table::get_sheet_height(table, Some(1))?;
        let mut mappings = HashMap::with_capacity(row);

        for r in 1..row {
            let key = table
                .cell_content(0, r)
                .ok_or::<crate::error::Error>("Get cell_content failed".into())?
                .trim();
            match mappings.entry(key.to_string()) {
                std::collections::hash_map::Entry::Vacant(e) => {
                    let val = table
                        .cell_content(1, r)
                        .ok_or::<crate::error::Error>("Get cell_content failed".into())?
                        .trim();
                    e.insert(val.parse::<i32>()?);
                }
                _ => {
                    return Err(format!("Table {} has duplicate key `{}`", name, key).into());
                }
            }
        }

        ctx.efks.insert(name.into(), mappings);
        Ok(Self { name })
    }
}
