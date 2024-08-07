use std::collections::BTreeMap;
use std::fmt::Write;

use super::{InnerBuildContext, Template};
use crate::util::format;
use crate::util::format_fmt;
use crate::{config::CFG, error::Error};

pub(crate) fn build<W: std::io::Write + ?Sized>(
    template: &Template<'_>,
    stream: &mut W,
    tab_nums: i32,
    ctx: &InnerBuildContext<'_>,
    is_server: bool,
) -> Result<(), Error> {
    if is_server {
        inner_build_server(template, stream, tab_nums, ctx)?;
    } else {
        inner_build_client(template, stream, tab_nums, ctx)?;
    }
    Ok(())
}

pub(crate) fn inner_build_client<W: std::io::Write + ?Sized>(
    template: &Template<'_>,
    stream: &mut W,
    tab_nums: i32,
    ctx: &InnerBuildContext<'_>,
) -> Result<(), Error> {
    let end = CFG.line_end_flag;
    let (id_type, lines) = unsafe {
        (
            ctx.values
                .get_unchecked(0)
                .first()
                .map(|v| v.ty_info().to_string()),
            ctx.values.get_unchecked(0).len(),
        )
    };
    let lines_to_switch = CFG.rows_to_switch;

    //--------------fixed code----------------------------
    format(tab_nums, stream)?;
    stream.write("[Serializable]".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums, stream)?;
    stream.write("public class ".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write(" : IEnumerable<".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item>, IConfigData".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("public static ".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write(" Instance = new ".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("();".as_bytes())?;
    stream.write(end.as_bytes())?;
    //--------------fixed code----------------------------

    // DefKey static class
    if let Some(ref vals) = ctx.keytypes {
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("public static class DefKey".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("{".as_bytes())?;
        stream.write(end.as_bytes())?;

        if template.raw_refs.is_empty() {
            for v in vals.iter().filter(|v| !v.0.is_empty()) {
                format(tab_nums + 2, stream)?;
                stream.write_fmt(format_args!(
                    "public const {} ",
                    id_type.as_ref().ok_or::<Error>(
                        format!("Can't find id type for {} table", template.name).into()
                    )?,
                ))?;
                stream.write(v.0.as_bytes())?;
                stream.write(" = ".as_bytes())?;
                stream.write_fmt(format_args!("{};{}", v.1, end))?;
            }
        } else {
            for v in vals.iter().filter(|v| !v.0.is_empty()) {
                if let Some(k) = template.raw_refs.get(v.2) {
                    format(tab_nums + 2, stream)?;
                    stream.write_fmt(format_args!(
                        "public const {} ",
                        id_type.as_ref().ok_or::<Error>(
                            format!("Can't find id type for {} table", template.name).into()
                        )?,
                    ))?;
                    stream.write(v.0.as_bytes())?;
                    stream.write(" = ".as_bytes())?;
                    stream.write_fmt(format_args!("{};{}", k, end))?;
                } else {
                    format(tab_nums + 2, stream)?;
                    stream.write_fmt(format_args!(
                        "public const {} ",
                        id_type.as_ref().ok_or::<Error>(
                            format!("Can't find id type for {} table", template.name).into()
                        )?,
                    ))?;
                    stream.write(v.0.as_bytes())?;
                    stream.write(" = ".as_bytes())?;
                    stream.write_fmt(format_args!("{};{}", v.1, end))?;
                }
            }
        }

        format(tab_nums + 1, stream)?;
        stream.write("}".as_bytes())?;
        stream.write(end.as_bytes())?;
    }

    format(tab_nums + 1, stream)?;
    stream.write(
        "private readonly Dictionary<string, int> _refNameMap = new Dictionary<string, int>();"
            .as_bytes(),
    )?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("public IReadOnlyDictionary<string, int> RefNameMap => _refNameMap;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("private List<".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item> _dataArray = null;".as_bytes())?;
    stream.write(end.as_bytes())?;

    let mut sort: BTreeMap<usize, String> = BTreeMap::new();
    sort.extend_reserve(template.main.row);
    let mut piece = String::new();
    #[allow(unused_assignments)]
    let mut tid = None;

    for row in 0..lines {
        format_fmt(tab_nums + 2, &mut piece)?;
        piece.push_str("_dataArray.Add(new ");
        piece.push_str(template.name);
        piece.push_str("Item(");

        if let Some(v) = template
            .raw_refs
            .get(unsafe { *ctx.templates.get_unchecked(row) })
        {
            tid = Some(*v as _);
            piece.write_fmt(format_args!("{}", v))?;
        } else {
            piece.write_fmt(format_args!("{}", row))?;
            tid = Some(row);
        }
        piece.push(',');

        for (idx, v) in ctx.required.iter().skip(1).enumerate() {
            let rows = unsafe { ctx.values.get_unchecked(v.0) };
            rows[row].code_fmt(&mut piece)?;
            if idx < ctx.required.len() - 2 {
                piece.push(',');
            }
        }

        piece.push_str("));");
        piece.push_str(end);
        sort.insert(unsafe { tid.unwrap_unchecked() }, piece);
        piece = String::new();
    }

    for term in 0..(lines / lines_to_switch) + (if lines % lines_to_switch == 0 { 0 } else { 1 }) {
        let idx = term * lines_to_switch;
        let end_idx = if lines - idx < lines_to_switch {
            lines
        } else {
            idx + lines_to_switch
        };

        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("private void CreateItems".as_bytes())?;
        stream.write(term.to_string().as_bytes())?;
        stream.write("()".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("{".as_bytes())?;
        stream.write(end.as_bytes())?;

        for row in idx..end_idx {
            if sort.contains_key(&row) {
                stream.write(sort[&row].as_bytes())?;
            }
        }

        format(tab_nums + 1, stream)?;
        stream.write("}".as_bytes())?;
        stream.write(end.as_bytes())?;
    }

    //--------------------------Init-begin----------------------------------
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("public void Init()".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("_refNameMap.Clear();".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("_refNameMap.Load(\"".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("\");".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("_extraDataMap.Clear();".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("_dataArray = new List<".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write_fmt(format_args!("Item>( {} ) {{{}", lines, end))?;
    format(tab_nums + 2, stream)?;
    stream.write("};".as_bytes())?;
    for term in 0..(lines / lines_to_switch) + (if lines % lines_to_switch == 0 { 0 } else { 1 }) {
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("CreateItems".as_bytes())?;
        stream.write(term.to_string().as_bytes())?;
        stream.write("();".as_bytes())?;
    }
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("}".as_bytes())?;
    stream.write(end.as_bytes())?;
    //--------------------------Init-end.as_bytes()----------------------------------

    //--------------------------GetItemId-begin----------------------------------
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("public int GetItemId(string refName)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("if (_refNameMap.TryGetValue(refName, out var id))".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("return id;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("throw new Exception($\"{refName} not found.\");".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("}".as_bytes())?;
    stream.write(end.as_bytes())?;
    //--------------------------GetItemId-end.as_bytes()----------------------------------

    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("private readonly Dictionary<int, ".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item> _extraDataMap = new Dictionary<int, ".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item>();".as_bytes())?;
    stream.write(end.as_bytes())?;
    stream.write(end.as_bytes())?;

    //--------------------------AddExtraItem-begin----------------------------------
    format(tab_nums + 1, stream)?;
    stream.write(
        "public int AddExtraItem(string identifier, string refName, object configItem)".as_bytes(),
    )?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("var item = (".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item)configItem;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("var id = (int) item.TemplateId;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("if (id < _dataArray.Count)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("throw new Exception($\"".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write(
        " template id {item.TemplateId} created by {identifier} already exist.\");".as_bytes(),
    )?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("if (_extraDataMap.ContainsKey(id))".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("throw new Exception($\"".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write(
        " extra template id {item.TemplateId} created by {identifier} already exist.\");"
            .as_bytes(),
    )?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("if (_refNameMap.TryGetValue(refName, out var refId))".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("throw new Exception($\"".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write(" template reference name {refName}(id = {item.TemplateId}) created by {identifier} already exist with templateId {refId}).\");".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("_refNameMap.Add(refName, id);".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("_extraDataMap.Add(id, item);".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("return id;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("}".as_bytes())?;
    stream.write(end.as_bytes())?;
    //--------------------------AddExtraItem-end.as_bytes()----------------------------------

    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("public ".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write_fmt(format_args!(
        "Item this[{} id] => GetItem(id);",
        id_type
            .as_ref()
            .ok_or::<Error>(format!("Can't find id type for {} table", template.name).into())?,
    ))?;
    stream.write(end.as_bytes())?;

    if id_type
        .as_ref()
        .ok_or::<Error>(format!("Can't find id type for {} table", template.name).into())?
        != "int"
    {
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("public ".as_bytes())?;
        stream.write(template.name.as_bytes())?;
        stream.write_fmt(format_args!(
            "Item this[int id] => GetItem(({})id);",
            id_type
                .as_ref()
                .ok_or::<Error>(format!("Can't find id type for {} table", template.name).into())?
        ))?;
        stream.write(end.as_bytes())?;
        stream.write(end.as_bytes())?;
    }

    //--------------------------GetItem-begin----------------------------------
    format(tab_nums + 1, stream)?;
    stream.write("public ".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write_fmt(format_args!(
        "Item GetItem({} id)",
        id_type
            .as_ref()
            .ok_or::<Error>(format!("Can't find id type for {} table", template.name).into())?
    ))?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("if (id < 0) return null;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("if (id < _dataArray.Count) return _dataArray[(int)id];".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream
        .write("if (_extraDataMap.TryGetValue((int) id, out var item)) return item;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("// 预期为有效 Id 但仍然访问不到数据时".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("GameData.Utilities.AdaptableLog.TagWarning(GetType().FullName, $\"index {id} is not in range [0, {_dataArray.Count}) and is not defined in _extraDataMap (count: {_extraDataMap.Count})\");".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("return null;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("}".as_bytes())?;
    stream.write(end.as_bytes())?;
    stream.write(end.as_bytes())?;
    //--------------------------GetItem-end.as_bytes()----------------------------------

    format(tab_nums + 1, stream)?;
    stream.write("public ".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item this[string refName] => this[_refNameMap[refName]];".as_bytes())?;
    stream.write(end.as_bytes())?;
    stream.write(end.as_bytes())?;

    // enum-flags
    for (k, _) in ctx.enumflags.iter() {
        format(tab_nums + 1, stream)?;
        stream.write_fmt(format_args!(
            "public static int Get{}Bonus(int key, E{}ReferencedType property){}",
            k, k, end
        ))?;
        format(tab_nums + 1, stream)?;
        stream.write("{".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write_fmt(format_args!(
            "return Instance._dataArray[key].Get{}BonusInt(property);{}",
            k, end
        ))?;
        format(tab_nums + 1, stream)?;
        stream.write("}".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write(end.as_bytes())?;

        format(tab_nums + 1, stream)?;
        stream.write_fmt(format_args!(
            "public static int Get{}Bonus(short[] keys, E{}ReferencedType property){}",
            k, k, end
        ))?;
        format(tab_nums + 1, stream)?;
        stream.write("{".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("int sum = 0;".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("for (int i = 0, count = keys.Length; i < count; ++i)".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 3, stream)?;
        stream.write_fmt(format_args!(
            "sum += Instance._dataArray[keys[i]].Get{}BonusInt(property);{}",
            k, end
        ))?;
        format(tab_nums + 2, stream)?;
        stream.write("return sum;".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("}".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write(end.as_bytes())?;

        format(tab_nums + 1, stream)?;
        stream.write_fmt(format_args!(
            "public static int Get{}Bonus(List<short> keys, E{}ReferencedType property){}",
            k, k, end
        ))?;
        format(tab_nums + 1, stream)?;
        stream.write("{".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("int sum = 0;".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("for (int i = 0, count = keys.Count; i < count; ++i)".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 3, stream)?;
        stream.write_fmt(format_args!(
            "sum += Instance._dataArray[keys[i]].Get{}BonusInt(property);{}",
            k, end
        ))?;
        format(tab_nums + 2, stream)?;
        stream.write("return sum;".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("}".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write(end.as_bytes())?;

        format(tab_nums + 1, stream)?;
        stream.write_fmt(format_args!(
            "public static int Get{}Bonus(int[] keys, E{}ReferencedType property){}",
            k, k, end
        ))?;
        format(tab_nums + 1, stream)?;
        stream.write("{".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("int sum = 0;".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("for (int i = 0, count = keys.Length; i < count; ++i)".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 3, stream)?;
        stream.write_fmt(format_args!(
            "sum += Instance._dataArray[keys[i]].Get{}BonusInt(property);{}",
            k, end
        ))?;
        format(tab_nums + 2, stream)?;
        stream.write("return sum;".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("}".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write(end.as_bytes())?;

        format(tab_nums + 1, stream)?;
        stream.write_fmt(format_args!(
            "public static int Get{}Bonus(List<int> keys, E{}ReferencedType property){}",
            k, k, end
        ))?;
        format(tab_nums + 1, stream)?;
        stream.write("{".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("int sum = 0;".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("for (int i = 0, count = keys.Count; i < count; ++i)".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 3, stream)?;
        stream.write_fmt(format_args!(
            "sum += Instance._dataArray[keys[i]].Get{}BonusInt(property);{}",
            k, end
        ))?;
        format(tab_nums + 2, stream)?;
        stream.write("return sum;".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("}".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write(end.as_bytes())?;
    }

    //--------------------------RequiredFields-begin----------------------------------
    format(tab_nums + 1, stream)?;
    stream.write(
        "private readonly HashSet<string> RequiredFields = new HashSet<string>()".as_bytes(),
    )?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;

    for v in template
        .fk_cols
        .iter()
        .map(|v| template.main.cell(*v, CFG.row_of_ident, true))
    {
        let v = v?;
        if !v.is_empty() {
            format(tab_nums + 2, stream)?;
            stream.write_fmt(format_args!("\"{}\",{}", v, end))?;
        }
    }

    for v in ctx.required.iter().filter(|v| !v.1.is_empty()) {
        if ctx.nodefs.contains(v.1) && !template.fk_cols.contains(&v.0) {
            format(tab_nums + 2, stream)?;
            stream.write_fmt(format_args!("\"{}\",{}", v.1, end))?;
        }
    }
    format(tab_nums + 1, stream)?;
    stream.write("};".as_bytes())?;
    stream.write(end.as_bytes())?;
    //--------------------------RequiredFields-end----------------------------------

    //--------------------------GetAllKeys-begin------------------------------------
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write_fmt(format_args!(
        "public List<{}> GetAllKeys()",
        id_type
            .as_ref()
            .ok_or::<Error>(format!("Can't find id type for {} table", template.name).into())?
    ))?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write_fmt(format_args!(
        "var keys = new List<{}>();",
        id_type
            .as_ref()
            .ok_or::<Error>(format!("Can't find id type for {} table", template.name).into())?
    ))?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write(
        "keys.AddRange(from item in _dataArray where null != item select item.TemplateId);"
            .as_bytes(),
    )?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("keys.AddRange(from item in _extraDataMap.Values where null != item select item.TemplateId);".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("return keys;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("}".as_bytes())?;
    stream.write(end.as_bytes())?;
    //--------------------------GetAllKeys-end--------------------------------------

    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("public int Count => _dataArray.Count;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("public int CountWithExtra => Count + _extraDataMap.Count;".as_bytes())?;
    stream.write(end.as_bytes())?;

    //--------------------------Iterate-begin----------------------------------
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("public void Iterate(Func<".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item,bool> iterateFunc)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("if(null == iterateFunc)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("return;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("foreach(".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item item in _dataArray)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("if(null == item)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 4, stream)?;
    stream.write("continue;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("if(!iterateFunc(item))".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 4, stream)?;
    stream.write("break;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("}".as_bytes())?;

    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("foreach(".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item item in _extraDataMap.Values)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("if(null == item)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 4, stream)?;
    stream.write("continue;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("if(!iterateFunc(item))".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 4, stream)?;
    stream.write("break;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("}".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("}".as_bytes())?;
    stream.write(end.as_bytes())?;
    //--------------------------Iterate-end.as_bytes()----------------------------------

    //--------------------------GetEnumerator-begin----------------------------------
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("IEnumerator<".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item> IEnumerable<".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item>.GetEnumerator()".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("foreach (var item in _dataArray)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("yield return item;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("foreach (var item in _extraDataMap.Values)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("yield return item;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("}".as_bytes())?;
    stream.write(end.as_bytes())?;

    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("IEnumerator IEnumerable.GetEnumerator()".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("foreach (var item in _dataArray)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("yield return item;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("foreach (var item in _extraDataMap.Values)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("yield return item;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("}".as_bytes())?;
    stream.write(end.as_bytes())?;
    //--------------------------GetEnumerator-end.as_bytes()----------------------------------
    format(tab_nums, stream)?;
    stream.write("}".as_bytes())?;
    Ok(())
}

pub(crate) fn inner_build_server<W: std::io::Write + ?Sized>(
    template: &Template<'_>,
    stream: &mut W,
    tab_nums: i32,
    ctx: &InnerBuildContext<'_>,
) -> Result<(), Error> {
    let end = CFG.line_end_flag;
    let (id_type, _) = unsafe {
        (
            ctx.values
                .get_unchecked(0)
                .first()
                .map(|v| v.ty_info().to_string()),
            ctx.values.get_unchecked(0).len(),
        )
    };

    //--------------fixed code----------------------------
    format(tab_nums, stream)?;
    stream.write("[Serializable]".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums, stream)?;
    stream.write("public class ".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write(" : IEnumerable<".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item>, IConfigData".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("public static ".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write(" Instance = new ".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("();".as_bytes())?;
    stream.write(end.as_bytes())?;
    //--------------fixed code----------------------------

    // DefKey static class
    if let Some(ref vals) = ctx.keytypes {
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("public static class DefKey".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("{".as_bytes())?;
        stream.write(end.as_bytes())?;

        if template.raw_refs.is_empty() {
            for v in vals.iter().filter(|v| !v.0.is_empty()) {
                format(tab_nums + 2, stream)?;
                stream.write_fmt(format_args!(
                    "public const {} ",
                    id_type.as_ref().ok_or::<Error>(
                        format!("Can't find id type for {} table", template.name).into()
                    )?,
                ))?;
                stream.write(v.0.as_bytes())?;
                stream.write(" = ".as_bytes())?;
                stream.write_fmt(format_args!("{};{}", v.1, end))?;
            }
        } else {
            for v in vals.iter().filter(|v| !v.0.is_empty()) {
                if let Some(k) = template.raw_refs.get(v.2) {
                    format(tab_nums + 2, stream)?;
                    stream.write_fmt(format_args!(
                        "public const {} ",
                        id_type.as_ref().ok_or::<Error>(
                            format!("Can't find id type for {} table", template.name).into()
                        )?,
                    ))?;
                    stream.write(v.0.as_bytes())?;
                    stream.write(" = ".as_bytes())?;
                    stream.write_fmt(format_args!("{};{}", k, end))?;
                } else {
                    format(tab_nums + 2, stream)?;
                    stream.write_fmt(format_args!(
                        "public const {} ",
                        id_type.as_ref().ok_or::<Error>(
                            format!("Can't find id type for {} table", template.name).into()
                        )?,
                    ))?;
                    stream.write(v.0.as_bytes())?;
                    stream.write(" = ".as_bytes())?;
                    stream.write_fmt(format_args!("{};{}", v.1, end))?;
                }
            }
        }

        format(tab_nums + 1, stream)?;
        stream.write("}".as_bytes())?;
        stream.write(end.as_bytes())?;
    }

    format(tab_nums + 1, stream)?;
    stream.write(
        "private readonly Dictionary<string, int> _refNameMap = new Dictionary<string, int>();"
            .as_bytes(),
    )?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("public IReadOnlyDictionary<string, int> RefNameMap => _refNameMap;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("private List<".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item> _dataArray = null;".as_bytes())?;
    stream.write(end.as_bytes())?;

    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("private readonly Dictionary<int, ".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item> _extraDataMap = new Dictionary<int, ".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item>();".as_bytes())?;
    stream.write(end.as_bytes())?;
    stream.write(end.as_bytes())?;

    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("public ".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write_fmt(format_args!(
        "Item this[{} id] => GetItem(id);",
        id_type
            .as_ref()
            .ok_or::<Error>(format!("Can't find id type for {} table", template.name).into())?,
    ))?;
    stream.write(end.as_bytes())?;

    if id_type
        .as_ref()
        .ok_or::<Error>(format!("Can't find id type for {} table", template.name).into())?
        != "int"
    {
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("public ".as_bytes())?;
        stream.write(template.name.as_bytes())?;
        stream.write_fmt(format_args!(
            "Item this[int id] => GetItem(({})id);",
            id_type
                .as_ref()
                .ok_or::<Error>(format!("Can't find id type for {} table", template.name).into())?
        ))?;
        stream.write(end.as_bytes())?;
        stream.write(end.as_bytes())?;
    }

    //--------------------------GetItem-begin----------------------------------
    format(tab_nums + 1, stream)?;
    stream.write("public ".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write_fmt(format_args!(
        "Item GetItem({} id)",
        id_type
            .as_ref()
            .ok_or::<Error>(format!("Can't find id type for {} table", template.name).into())?
    ))?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("if (id < 0) return null;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("if (id < _dataArray.Count) return _dataArray[(int)id];".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream
        .write("if (_extraDataMap.TryGetValue((int) id, out var item)) return item;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("// 预期为有效 Id 但仍然访问不到数据时".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("GameData.Utilities.AdaptableLog.TagWarning(GetType().FullName, $\"index {id} is not in range [0, {_dataArray.Count}) and is not defined in _extraDataMap (count: {_extraDataMap.Count})\");".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("return null;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("}".as_bytes())?;
    stream.write(end.as_bytes())?;
    stream.write(end.as_bytes())?;
    //--------------------------GetItem-end.as_bytes()----------------------------------

    format(tab_nums + 1, stream)?;
    stream.write("public ".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item this[string refName] => this[_refNameMap[refName]];".as_bytes())?;
    stream.write(end.as_bytes())?;
    stream.write(end.as_bytes())?;

    // enum-flags
    for (k, _) in ctx.enumflags.iter() {
        format(tab_nums + 1, stream)?;
        stream.write_fmt(format_args!(
            "public static int Get{}Bonus(int key, E{}ReferencedType property){}",
            k, k, end
        ))?;
        format(tab_nums + 1, stream)?;
        stream.write("{".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write_fmt(format_args!(
            "return Instance._dataArray[key].Get{}BonusInt(property);{}",
            k, end
        ))?;
        format(tab_nums + 1, stream)?;
        stream.write("}".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write(end.as_bytes())?;

        format(tab_nums + 1, stream)?;
        stream.write_fmt(format_args!(
            "public static int Get{}Bonus(short[] keys, E{}ReferencedType property){}",
            k, k, end
        ))?;
        format(tab_nums + 1, stream)?;
        stream.write("{".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("int sum = 0;".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("for (int i = 0, count = keys.Length; i < count; ++i)".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 3, stream)?;
        stream.write_fmt(format_args!(
            "sum += Instance._dataArray[keys[i]].Get{}BonusInt(property);{}",
            k, end
        ))?;
        format(tab_nums + 2, stream)?;
        stream.write("return sum;".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("}".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write(end.as_bytes())?;

        format(tab_nums + 1, stream)?;
        stream.write_fmt(format_args!(
            "public static int Get{}Bonus(List<short> keys, E{}ReferencedType property){}",
            k, k, end
        ))?;
        format(tab_nums + 1, stream)?;
        stream.write("{".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("int sum = 0;".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("for (int i = 0, count = keys.Count; i < count; ++i)".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 3, stream)?;
        stream.write_fmt(format_args!(
            "sum += Instance._dataArray[keys[i]].Get{}BonusInt(property);{}",
            k, end
        ))?;
        format(tab_nums + 2, stream)?;
        stream.write("return sum;".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("}".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write(end.as_bytes())?;

        format(tab_nums + 1, stream)?;
        stream.write_fmt(format_args!(
            "public static int Get{}Bonus(int[] keys, E{}ReferencedType property){}",
            k, k, end
        ))?;
        format(tab_nums + 1, stream)?;
        stream.write("{".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("int sum = 0;".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("for (int i = 0, count = keys.Length; i < count; ++i)".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 3, stream)?;
        stream.write_fmt(format_args!(
            "sum += Instance._dataArray[keys[i]].Get{}BonusInt(property);{}",
            k, end
        ))?;
        format(tab_nums + 2, stream)?;
        stream.write("return sum;".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("}".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write(end.as_bytes())?;

        format(tab_nums + 1, stream)?;
        stream.write_fmt(format_args!(
            "public static int Get{}Bonus(List<int> keys, E{}ReferencedType property){}",
            k, k, end
        ))?;
        format(tab_nums + 1, stream)?;
        stream.write("{".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("int sum = 0;".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("for (int i = 0, count = keys.Count; i < count; ++i)".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 3, stream)?;
        stream.write_fmt(format_args!(
            "sum += Instance._dataArray[keys[i]].Get{}BonusInt(property);{}",
            k, end
        ))?;
        format(tab_nums + 2, stream)?;
        stream.write("return sum;".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("}".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write(end.as_bytes())?;
    }

    //--------------------------GetAllKeys-begin------------------------------------
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write_fmt(format_args!(
        "public List<{}> GetAllKeys()",
        id_type
            .as_ref()
            .ok_or::<Error>(format!("Can't find id type for {} table", template.name).into())?
    ))?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write_fmt(format_args!(
        "var keys = new List<{}>();",
        id_type
            .as_ref()
            .ok_or::<Error>(format!("Can't find id type for {} table", template.name).into())?
    ))?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write(
        "keys.AddRange(from item in _dataArray where null != item select item.TemplateId);"
            .as_bytes(),
    )?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("keys.AddRange(from item in _extraDataMap.Values where null != item select item.TemplateId);".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("return keys;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("}".as_bytes())?;
    stream.write(end.as_bytes())?;
    //--------------------------GetAllKeys-end--------------------------------------

    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("public int Count => _dataArray.Count;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("public int CountWithExtra => Count + _extraDataMap.Count;".as_bytes())?;
    stream.write(end.as_bytes())?;

    //--------------------------Iterate-begin----------------------------------
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("public void Iterate(Func<".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item,bool> iterateFunc)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("if(null == iterateFunc)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("return;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("foreach(".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item item in _dataArray)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("if(null == item)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 4, stream)?;
    stream.write("continue;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("if(!iterateFunc(item))".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 4, stream)?;
    stream.write("break;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("}".as_bytes())?;

    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("foreach(".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item item in _extraDataMap.Values)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("if(null == item)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 4, stream)?;
    stream.write("continue;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("if(!iterateFunc(item))".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 4, stream)?;
    stream.write("break;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("}".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("}".as_bytes())?;
    stream.write(end.as_bytes())?;
    //--------------------------Iterate-end.as_bytes()----------------------------------

    //--------------------------GetEnumerator-begin----------------------------------
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("IEnumerator<".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item> IEnumerable<".as_bytes())?;
    stream.write(template.name.as_bytes())?;
    stream.write("Item>.GetEnumerator()".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("foreach (var item in _dataArray)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("yield return item;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("foreach (var item in _extraDataMap.Values)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("yield return item;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("}".as_bytes())?;
    stream.write(end.as_bytes())?;

    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("IEnumerator IEnumerable.GetEnumerator()".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("foreach (var item in _dataArray)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("yield return item;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 2, stream)?;
    stream.write("foreach (var item in _extraDataMap.Values)".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 3, stream)?;
    stream.write("yield return item;".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("}".as_bytes())?;
    stream.write(end.as_bytes())?;
    //--------------------------GetEnumerator-end.as_bytes()----------------------------------
    format(tab_nums, stream)?;
    stream.write("}".as_bytes())?;
    Ok(())
}
