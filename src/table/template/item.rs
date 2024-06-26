use crate::preconfig::PRECONFIG;
use crate::util::{self, format};
use crate::{config::CFG, error::Error};

use super::{InnerBuildContext, Template};

pub(crate) fn build<W: std::io::Write>(
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

pub(crate) fn inner_build_client<W: std::io::Write>(
    template: &Template<'_>,
    stream: &mut W,
    tab_nums: i32,
    ctx: &InnerBuildContext<'_>,
) -> Result<(), Error> {
    let end = CFG.line_end_flag;
    #[allow(unused_assignments)]
    let mut count = 0;
    let base_name = format!("{}Item", template.name);
    let comment = |content: &str, stream: &mut W| -> Result<(), Error> {
        format(tab_nums + 1, stream)?;
        stream.write("/// <summary>".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("/// ".as_bytes())?;
        stream.write(content.as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("/// </summary>".as_bytes())?;
        stream.write(end.as_bytes())?;
        Ok(())
    };

    format(tab_nums, stream)?;
    stream.write("[Serializable]".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums, stream)?;
    stream.write("public class ".as_bytes())?;
    stream.write(base_name.as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;

    for item in ctx.items.iter() {
        if !item.0.is_empty() {
            comment(item.0, stream)?;
        }

        format(tab_nums + 1, stream)?;
        stream.write("public readonly ".as_bytes())?;
        let mut s = item.2.to_string();
        convert_type(&mut s);

        if s == "enum" {
            stream.write_fmt(format_args!("E{}{}", template.name, item.1))?;
        } else {
            stream.write(replace_lstring(&s).as_bytes())?;
        }

        stream.write(" ".as_bytes())?;
        stream.write(item.1.as_bytes())?;
        stream.write(";".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write(end.as_bytes())?;
    }

    // construct_0-------------
    format(tab_nums + 1, stream)?;
    stream.write("public ".as_bytes())?;
    stream.write(base_name.as_bytes())?;
    stream.write("(".as_bytes())?;

    for item in ctx.items.iter() {
        let rows = unsafe { ctx.values.get_unchecked(item.3) };
        if !rows.is_empty() {
            stream.write_fmt(format_args!("{}", rows[0].ty_info()))?;
        }
        stream.write(" arg".as_bytes())?;
        stream.write(count.to_string().as_bytes())?;
        if count < ctx.items.len() - 1 {
            stream.write(",".as_bytes())?;
        }
        count += 1;
    }

    stream.write(")".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;

    // extra config----------
    if let Some(cfg) = PRECONFIG.get(template.name) {
        if !cfg.ctor_begin.is_empty() {
            stream.write(cfg.ctor_begin.as_bytes())?;
        }
    }
    // extra config----------

    count = 0;
    for (_, ident, _, col) in ctx.items.iter() {
        if !ident.is_empty() {
            format(tab_nums + 2, stream)?;
            stream.write(ident.as_bytes())?;

            let rows = unsafe { ctx.values.get_unchecked(*col) };
            if !rows.is_empty() {
                let info = rows[0].ty_info();
                if info.is_lstring() {
                    stream.write_fmt(format_args!(
                        " = LocalStringManager.GetConfig(\"{}_language\", arg{})",
                        template.name, count
                    ))?;
                } else if info.is_lstring_arr() {
                    stream.write_fmt(format_args!(
                        " = LocalStringManager.ConvertConfigList(\"{}_language\", arg{})",
                        template.name, count
                    ))?;
                } else {
                    stream.write_fmt(format_args!(" = arg{}", count))?;
                }
            }

            stream.write(";".as_bytes())?;
            stream.write(end.as_bytes())?;
            count += 1;
        }
    }

    format(tab_nums + 1, stream)?;
    stream.write("}".as_bytes())?;
    stream.write(end.as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums + 1, stream)?;
    // construct_0-------------

    // construct_1-------------
    stream.write("public ".as_bytes())?;
    stream.write(base_name.as_bytes())?;
    stream.write("()".as_bytes())?;
    stream.write(end.as_bytes())?;

    format(tab_nums + 1, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;

    count = 0;
    for (_, ident, _, _) in ctx.items.iter() {
        format(tab_nums + 2, stream)?;
        stream.write(ident.as_bytes())?;

        if let Some((info, val)) = ctx.defaults.get(ident) {
            stream.write(" = ".as_bytes())?;

            if info.is_lstring() {
                stream.write_fmt(format_args!(
                    "LocalStringManager.GetConfig(\"{}_language\", default)",
                    template.name
                ))?;
            } else if info.is_lstring_arr() {
                stream.write_fmt(format_args!(
                    "LocalStringManager.ConvertConfigList(\"{}_language\", default)",
                    template.name
                ))?;
            } else {
                unsafe { val.as_ref().unwrap_unchecked().code(stream) }?;
            }
            stream.write(";".as_bytes())?;
        } else {
            stream.write(" = default;".as_bytes())?;
        }
        stream.write(end.as_bytes())?;
        count += 1;
    }

    format(tab_nums + 1, stream)?;
    stream.write("}".as_bytes())?;
    stream.write(end.as_bytes())?;
    // construct_1-------------

    // enums
    for (k, arr) in ctx.enumflags.iter() {
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write_fmt(format_args!(
            "public int Get{}BonusInt(E{}ReferencedType key){}",
            k, k, end
        ))?;
        format(tab_nums + 1, stream)?;
        stream.write("{".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write_fmt(format_args!("switch (key){}", end))?;
        format(tab_nums + 2, stream)?;
        stream.write("{".as_bytes())?;
        stream.write(end.as_bytes())?;

        for v in arr {
            format(tab_nums + 3, stream)?;
            stream.write_fmt(format_args!(
                "case E{}ReferencedType.{}:return {};{}",
                k, v, v, end
            ))?;
        }

        format(tab_nums + 2, stream)?;
        stream.write("}".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("return 0;".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("}".as_bytes())?;
        stream.write(end.as_bytes())?;
    }

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
    #[allow(unused_assignments)]
    let base_name = format!("{}Item", template.name);
    let comment = |content: &str, stream: &mut W| -> Result<(), Error> {
        format(tab_nums + 1, stream)?;
        stream.write("/// <summary>".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("/// ".as_bytes())?;
        stream.write(content.as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("/// </summary>".as_bytes())?;
        stream.write(end.as_bytes())?;
        Ok(())
    };

    format(tab_nums, stream)?;
    stream.write("[Serializable]".as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums, stream)?;
    stream.write("public class ".as_bytes())?;
    stream.write(base_name.as_bytes())?;
    stream.write(end.as_bytes())?;
    format(tab_nums, stream)?;
    stream.write("{".as_bytes())?;
    stream.write(end.as_bytes())?;

    for item in ctx.items.iter() {
        if !item.0.is_empty() {
            comment(item.0, stream)?;
        }

        format(tab_nums + 1, stream)?;
        stream.write("public readonly ".as_bytes())?;
        let mut s = item.2.to_string();
        convert_type(&mut s);

        if s == "enum" {
            stream.write_fmt(format_args!("E{}{}", template.name, item.1))?;
        } else {
            stream.write(replace_lstring(&s).as_bytes())?;
        }

        stream.write(" ".as_bytes())?;
        stream.write(item.1.as_bytes())?;
        stream.write(";".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write(end.as_bytes())?;
    }

    // enums
    for (k, arr) in ctx.enumflags.iter() {
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write_fmt(format_args!(
            "public int Get{}BonusInt(E{}ReferencedType key){}",
            k, k, end
        ))?;
        format(tab_nums + 1, stream)?;
        stream.write("{".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write_fmt(format_args!("switch (key){}", end))?;
        format(tab_nums + 2, stream)?;
        stream.write("{".as_bytes())?;
        stream.write(end.as_bytes())?;

        for v in arr {
            format(tab_nums + 3, stream)?;
            stream.write_fmt(format_args!(
                "case E{}ReferencedType.{}:return {};{}",
                k, v, v, end
            ))?;
        }

        format(tab_nums + 2, stream)?;
        stream.write("}".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 2, stream)?;
        stream.write("return 0;".as_bytes())?;
        stream.write(end.as_bytes())?;
        format(tab_nums + 1, stream)?;
        stream.write("}".as_bytes())?;
        stream.write(end.as_bytes())?;
    }

    format(tab_nums, stream)?;
    stream.write("}".as_bytes())?;
    Ok(())
}

fn convert_type(v: &mut String) {
    if let Some(idx) = v.find('[') {
        let mut n = idx;
        while let Some(c) = v.chars().nth(n) {
            if c == ']' {
                break;
            } else {
                n = n + 1;
            }
        }
        v.replace_range(idx + 1..n, "");
    }
}

fn replace_lstring(val: &str) -> String {
    let mut ret = String::with_capacity(val.len());
    let indexs_1 = util::bm_search(val, "LString");
    let indexs_2 = util::bm_search(val, "Lstring");

    if indexs_1.is_empty() && indexs_2.is_empty() {
        return String::from(val);
    } else if indexs_1.is_empty() {
        if indexs_2[0] == 0 {
            ret.push_str("string");
            ret.push_str(&val[7..]);
        } else {
            ret.push_str(&val[..indexs_2[0]]);
            ret.push('s');
            ret.push_str(&val[indexs_2[0] + 2..]);
        }
    } else if indexs_2.is_empty() {
        if indexs_1[0] == 0 {
            ret.push_str("string");
            ret.push_str(&val[7..]);
        } else {
            ret.push_str(&val[..indexs_1[0]]);
            ret.push('s');
            ret.push_str(&val[indexs_1[0] + 2..]);
        }
    } else {
        unreachable!()
    }
    ret
}
