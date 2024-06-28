#![allow(dead_code)]

use std::{
    cell::Cell, collections::HashMap, hint::unreachable_unchecked, ops::Neg, str::FromStr,
    sync::LazyLock,
};

use vnlex::{
    cursor::Cursor,
    lexer::{RawBuilder, RawLexer},
    syntaxer::Syntaxer,
    token::{tokenizers, Tokenizer},
};

use crate::{
    error,
    lex::{
        integer,
        states::{
            self,
            nodes::{
                array_elements, array_type, assign, custom_type, enum_type, integer_literal,
                list_type, literal_vals, tuple_type, tuple_type_elements, value_tuple_type,
                value_type, values,
            },
        },
        CData,
    },
    types::{
        custom::Custom,
        numbers::{Bool, Byte, Decimal, Double, Float, Int, SByte, Short, UInt, UShort},
        r#enum::Enum,
        sequence::{Array, FixedArray, List, ShortList, Tuple, ValueTuple},
        string::{LString, SString},
        TypeInfo, Value,
    },
    util,
};

struct Context<'a> {
    ls_emptys: Option<&'a Vec<i32>>,
    ls_map: Option<&'a HashMap<String, i32>>,
    current_idx: Cell<usize>,
}

pub struct Parser {
    lexer: RawLexer<dyn for<'a> Tokenizer<'a, CData<'a>, ()> + Send + Sync>,
    syntaxer: Syntaxer<'static, states::ReductionType>,
}

impl Parser {
    fn new() -> Self {
        let lexer = RawBuilder::whitespace()
            .append(tokenizers::QuotedString::<'"'>)
            .append(integer::Integer)
            .append(tokenizers::symbol_with_sorted_array(states::DEF_SYMBOLS))
            .append(tokenizers::identifier_keyword_with_sorted_array(
                states::DEF_KEYWORDS,
            ))
            .build();
        Self {
            lexer,
            syntaxer: Syntaxer::new(states::DEF_STATES),
        }
    }
}

pub static PARSER: LazyLock<Parser> = LazyLock::new(Parser::new);

pub fn parse_assign_with_type(
    ty: &Box<value_type>,
    vals: &str,
    ls_map: Option<&HashMap<String, i32>>,
    ls_emptys: Option<&Vec<i32>>,
) -> Result<Box<dyn Value>, error::Error> {
    let ctx = Context {
        ls_map,
        ls_emptys,
        current_idx: Cell::new(0),
    };
    let tyinfo = get_value_type(ty)?;

    let vals = if vals.is_empty() {
        match tyinfo {
            TypeInfo::Int | TypeInfo::SByte | TypeInfo::Short | TypeInfo::Bool => "-1",
            TypeInfo::Array(_)
            | TypeInfo::Enum(_, _)
            | TypeInfo::List(_)
            | TypeInfo::FixedArray(_, _)
            | TypeInfo::Custom(_) => "null",
            TypeInfo::Double | TypeInfo::Float => "0.0",
            TypeInfo::Decimal | TypeInfo::UShort | TypeInfo::Uint | TypeInfo::Byte => "0",
            _ => "",
        }
    } else if tyinfo == TypeInfo::String && vals.len() == 2 {
        "null"
    } else {
        vals
    };

    let box_vals = parse_value(vals, 0, 0)?;
    match get_value(ty, &box_vals, &ctx) {
        Ok(e) => Ok(e),
        e => e.map_err(|e| format!("vals = `{}`, error: {}", vals, e).into()),
    }
}

#[deprecated]
pub fn parse_assign(
    expr: &str,
    ls_map: Option<&HashMap<String, i32>>,
    ls_emptys: Option<&Vec<i32>>,
) -> Result<Box<dyn Value>, error::Error> {
    let parser = &*PARSER;
    let mut cursor = Cursor::new(expr, 0, 0, None);
    let assign = parser
        .syntaxer
        .parse_optional::<_, _, assign>(parser.lexer.tokenizing(&mut cursor, &mut ()))
        .map_err(|e| e.into(&cursor))?
        .ok_or(error::Error::from("parse_optional return none"))?;
    let ctx = Context {
        ls_map,
        ls_emptys,
        current_idx: Cell::new(0),
    };
    match assign.as_ref() {
        assign::p0(ty, _, vals) => get_value(ty, vals, &ctx),
        assign::p1(ty, _, vals) => get_value(ty, vals, &ctx),
    }
}

#[allow(dead_code)]
pub fn parse_type(expr: &str, row: usize, col: usize) -> Result<Box<value_type>, error::Error> {
    let parser = &*PARSER;
    let mut cursor = Cursor::new(expr, row, col, None);
    let ty = parser
        .syntaxer
        .parse_optional::<_, _, value_type>(parser.lexer.tokenizing(&mut cursor, &mut ()))
        .map_err(|e| e.into(&cursor))?
        .ok_or(error::Error::from("parse_optional return none"))?;
    Ok(ty)
}

#[allow(dead_code)]
pub fn parse_value(expr: &str, row: usize, col: usize) -> Result<Box<values>, error::Error> {
    let parser = &*PARSER;
    let mut cursor = Cursor::new(expr, row, col, None);
    let val = parser
        .syntaxer
        .parse_optional::<_, _, values>(parser.lexer.tokenizing(&mut cursor, &mut ()))
        .map_err(|e| e.into(&cursor))?
        .ok_or(error::Error::from("parse_optional return none"))?;
    Ok(val)
}

pub fn get_value_type(ty: &Box<value_type>) -> Result<TypeInfo, error::Error> {
    match ty.as_ref() {
        value_type::p0(_) => Ok(TypeInfo::Decimal),
        value_type::p1(_) => Ok(TypeInfo::Float),
        value_type::p2(_) => Ok(TypeInfo::Double),
        value_type::p3(_) => Ok(TypeInfo::Int),
        value_type::p4(_) => Ok(TypeInfo::Uint),
        value_type::p5(_) => Ok(TypeInfo::Short),
        value_type::p6(_) => Ok(TypeInfo::UShort),
        value_type::p7(_) => Ok(TypeInfo::LString),
        value_type::p8(v) => parse_array_type(v),
        value_type::p9(v) => parse_list_type(v),
        value_type::p10(_) => Ok(TypeInfo::ShortList),
        value_type::p11(_) => Ok(TypeInfo::String),
        value_type::p12(v) => parse_value_tuple_type(v),
        value_type::p13(_) => Ok(TypeInfo::Bool),
        value_type::p14(v) => parse_custom_type(v),
        value_type::p15(v) => parse_enum_type(v),
        value_type::p16(v) => parse_tuple_type(v),
        value_type::p17(_) => Ok(TypeInfo::Byte),
        value_type::p18(_) => Ok(TypeInfo::SByte),
    }
}

fn parse_enum_type(ty: &Box<enum_type>) -> Result<TypeInfo, error::Error> {
    match ty.as_ref() {
        enum_type::p0(_, base, _, name) | enum_type::p1(_, base, _, name) => Ok(TypeInfo::Enum(
            base.as_ref().0.content.into(),
            name.as_ref().0.content.into(),
        )),
    }
}

fn parse_custom_type(ty: &Box<custom_type>) -> Result<TypeInfo, error::Error> {
    let mut ident = String::with_capacity(16);
    parse_custom_type_inner(ty, &mut ident)?;
    Ok(TypeInfo::Custom(ident))
}

fn parse_custom_type_inner(ty: &Box<custom_type>, ident: &mut String) -> Result<(), error::Error> {
    match ty.as_ref() {
        custom_type::p0(v) => ident.push_str(v.as_ref().0.content),
        custom_type::p1(prev, _, v) => {
            parse_custom_type_inner(prev, ident)?;
            ident.push('.');
            ident.push_str(v.as_ref().0.content);
        }
    }
    Ok(())
}

fn parse_list_type(ty: &Box<list_type>) -> Result<TypeInfo, error::Error> {
    let list_type::p0(_, _, ty, _) = ty.as_ref();
    Ok(TypeInfo::List(Box::new(get_value_type(ty)?)))
}

fn parse_array_type(ty: &Box<array_type>) -> Result<TypeInfo, error::Error> {
    match ty.as_ref() {
        array_type::p0(ty, _, _) => Ok(TypeInfo::Array(Box::new(get_value_type(ty)?))),
        array_type::p1(ty, _, nums, _) => Ok(TypeInfo::FixedArray(
            Box::new(get_value_type(ty)?),
            get_non_neg_integer_value(nums)?,
        )),
    }
}

fn parse_value_tuple_type(ty: &Box<value_tuple_type>) -> Result<TypeInfo, error::Error> {
    let ty = match ty.as_ref() {
        value_tuple_type::p0(_, _, ty, _) => ty,
        value_tuple_type::p1(_, _, ty, _, _) => ty,
    };
    let mut types = Vec::new();
    parse_tuple_type_inner(ty, &mut types)?;
    Ok(TypeInfo::ValueTuple(types))
}

fn parse_tuple_type(ty: &Box<tuple_type>) -> Result<TypeInfo, error::Error> {
    let ty = match ty.as_ref() {
        tuple_type::p0(_, _, ty, _) => ty,
        tuple_type::p1(_, _, ty, _, _) => ty,
    };
    let mut types = Vec::new();
    parse_tuple_type_inner(ty, &mut types)?;
    Ok(TypeInfo::Tuple(types))
}

fn parse_tuple_type_inner(
    ty: &Box<tuple_type_elements>,
    types: &mut Vec<Box<TypeInfo>>,
) -> Result<(), error::Error> {
    match ty.as_ref() {
        states::nodes::tuple_type_elements::p0(v) => types.push(Box::new(get_value_type(v)?)),
        states::nodes::tuple_type_elements::p1(prev, _, v) => {
            parse_tuple_type_inner(prev, types)?;
            types.push(Box::new(get_value_type(v)?));
        }
    }
    Ok(())
}

fn get_value(
    ty: &Box<value_type>,
    vals: &Box<values>,
    ctx: &Context,
) -> Result<Box<dyn Value>, error::Error> {
    let type_info = get_value_type(ty)?;
    match ty.as_ref() {
        value_type::p0(_) => parse_decimal_value(type_info, vals),
        value_type::p1(_) => parse_float_value(type_info, vals),
        value_type::p2(_) => parse_double_value(type_info, vals),
        value_type::p3(_) => parse_int_value(type_info, vals),
        value_type::p4(_) => parse_uint_value(type_info, vals),
        value_type::p5(_) => parse_short_value(type_info, vals),
        value_type::p6(_) => parse_ushort_value(type_info, vals),
        value_type::p7(_) => parse_lstring_value(type_info, vals, &ctx),
        value_type::p8(v) => parse_array_value(v, type_info, vals, ctx),
        value_type::p9(v) => parse_list_value(v, type_info, vals, ctx),
        value_type::p10(_) => parse_shortlist_value(type_info, vals, ctx),
        value_type::p11(_) => parse_string_value(type_info, vals),
        value_type::p12(v) => parse_valuetuple_value(v, type_info, vals, ctx),
        value_type::p13(_) => parse_bool_value(type_info, vals),
        value_type::p14(_) => parse_custom_value(type_info, vals),
        value_type::p15(_) => parse_enum_value(type_info, vals),
        value_type::p16(v) => parse_tuple_value(v, type_info, vals, ctx),
        value_type::p17(_) => parse_byte_value(type_info, vals),
        value_type::p18(_) => parse_sbyte_value(type_info, vals),
    }
}

trait ConstValue<T> {
    const MIN: T;
    const MAX: T;
}

impl ConstValue<i16> for i16 {
    const MIN: i16 = i16::MIN;
    const MAX: i16 = i16::MAX;
}

impl ConstValue<i32> for i32 {
    const MIN: i32 = i32::MIN;
    const MAX: i32 = i32::MAX;
}

impl ConstValue<i64> for i64 {
    const MIN: i64 = i64::MIN;
    const MAX: i64 = i64::MAX;
}

impl ConstValue<i8> for i8 {
    const MIN: i8 = i8::MIN;
    const MAX: i8 = i8::MAX;
}

impl ConstValue<f32> for f32 {
    const MIN: f32 = f32::MIN;
    const MAX: f32 = f32::MAX;
}

impl ConstValue<f64> for f64 {
    const MIN: f64 = f64::MIN;
    const MAX: f64 = f64::MAX;
}

fn get_number_value<T: FromStr + Neg<Output = T> + ConstValue<T>>(
    val: &Box<integer_literal>,
) -> Result<T, error::Error> {
    match val.as_ref() {
        states::nodes::integer_literal::p0(_) => todo!(),
        states::nodes::integer_literal::p1(_, _) => todo!(),
        states::nodes::integer_literal::p2(_) => todo!(),
        states::nodes::integer_literal::p3(_, _) => todo!(),
        states::nodes::integer_literal::p4(v) => {
            let (_, minus) = v
                .as_ref()
                .0
                .data
                .get_custom()
                .ok_or::<error::Error>("get custom cdata failed".into())?
                .into_digits()
                .ok_or::<error::Error>("".into())?;
            if minus {
                Ok(-v.0.content.parse::<T>().map_err::<error::Error, _>(|_| {
                    format!("parse number value failed: `-{}`", v.as_ref().0.content).into()
                })?)
            } else {
                Ok(v.0.content.parse::<T>().map_err::<error::Error, _>(|_| {
                    format!("parse number value failed: `{}`", v.as_ref().0.content).into()
                })?)
            }
        }
        states::nodes::integer_literal::p5(_, _) => todo!(),
        states::nodes::integer_literal::p6(_) => todo!(),
        states::nodes::integer_literal::p7(_, _) => todo!(),
        states::nodes::integer_literal::p8(ty, _, val) => {
            if ty.as_ref().0.content == "short" {
                match val.as_ref().0.content {
                    "MinValue" => Ok(T::MIN),
                    _ => unsafe { unreachable_unchecked() },
                }
            } else {
                unsafe { unreachable_unchecked() }
            }
        }
    }
}

fn get_non_neg_integer_value<T: FromStr>(val: &Box<integer_literal>) -> Result<T, error::Error> {
    match val.as_ref() {
        states::nodes::integer_literal::p0(_) => todo!(),
        states::nodes::integer_literal::p1(_, _) => todo!(),
        states::nodes::integer_literal::p2(_) => todo!(),
        states::nodes::integer_literal::p3(_, _) => todo!(),
        states::nodes::integer_literal::p4(v) => {
            Ok(v.0.content.parse::<T>().map_err::<error::Error, _>(|_| {
                format!("parse integer value failed: `{}`", v.as_ref().0.content).into()
            })?)
        }
        states::nodes::integer_literal::p5(_, _) => todo!(),
        states::nodes::integer_literal::p6(_) => todo!(),
        states::nodes::integer_literal::p7(_, _) => todo!(),
        states::nodes::integer_literal::p8(_, _, _) => todo!(),
    }
}

fn parse_lstring_value(
    ty: TypeInfo,
    vals: &Box<values>,
    ctx: &Context,
) -> Result<Box<dyn Value>, error::Error> {
    let values::p0(literal) = vals.as_ref() else {
        return Err("expected literal for lstring value".into());
    };
    let literal_vals::p3(raw) = literal.as_ref() else {
        return Err("expected string for lstring value".into());
    };

    let raw = raw.as_ref().0.content;
    if raw.len() < 2 {
        return Err("raw string val is not long enough".into());
    }
    let raw = &raw[1..raw.len() - 1];
    let idx = {
        if raw.is_empty() {
            match ctx.ls_emptys.as_ref() {
                Some(emptys) => {
                    if ctx.current_idx.get() >= emptys.len() {
                        return Err("Index overflow when find empty lstring value".into());
                    }
                    let val = emptys[ctx.current_idx.get()];
                    ctx.current_idx.update(|v| v + 1);
                    val
                }
                None => -1,
            }
        } else {
            let mapping = ctx.ls_map.as_ref().ok_or::<error::Error>(
                "Can't find lstring mapping when parse lstring value".into(),
            )?;
            **mapping.get(raw).as_ref().ok_or::<error::Error>(
                format!("Can't find lstring mapping when parse `{}`", raw).into(),
            )?
        }
    };

    Ok(Box::new(LString { ty, idx }) as _)
}

fn parse_decimal_value(ty: TypeInfo, vals: &Box<values>) -> Result<Box<dyn Value>, error::Error> {
    let values::p0(_) = vals.as_ref() else {
        return Err("expected literal_value for decimal type".into());
    };
    let raw = get_raw_value(vals)?;
    let Ok(real) = rust_decimal::Decimal::from_str_exact(&raw) else {
        return Err(format!("convert `{}` to decimal failed", raw).into());
    };
    Ok(Box::new(Decimal { ty, val: real }) as _)
}

fn parse_shortlist_value(
    ty: TypeInfo,
    vals: &Box<values>,
    ctx: &Context,
) -> Result<Box<dyn Value>, error::Error> {
    if let values::p3(_) = vals.as_ref() {
        return Ok(Box::new(ShortList {
            ty,
            vals: Vec::new(),
            is_null: true,
        }) as _);
    }

    let values::p1(array_vals) = vals.as_ref() else {
        return Err("expected array_vals for ShortList".into());
    };
    match parse_type("List<short>", 0, 0)?.as_ref() {
        value_type::p9(list_ty) => {
            let list_type::p0(_, _, raw, _) = list_ty.as_ref();
            let mut vals = Vec::new();
            match array_vals.as_ref() {
                states::nodes::array_vals::p0(_, _) => {
                    return Ok(Box::new(ShortList {
                        ty,
                        vals: Vec::new(),
                        is_null: false,
                    }) as _)
                }
                states::nodes::array_vals::p1(_, elements, _) => {
                    parse_array_elements_value(&raw, elements, &mut vals, ctx)?
                }
                states::nodes::array_vals::p2(_, elements, _, _) => {
                    parse_array_elements_value(&raw, elements, &mut vals, ctx)?
                }
            }
            Ok(Box::new(ShortList {
                ty,
                vals,
                is_null: false,
            }) as _)
        }
        _ => return Err("Parse ShortList type failed".into()),
    }
}

fn parse_enum_value(ty: TypeInfo, vals: &Box<values>) -> Result<Box<dyn Value>, error::Error> {
    if let values::p3(_) = vals.as_ref() {
        return Ok(Box::new(Enum {
            ty,
            ident: "".into(),
            is_null: true,
        }) as _);
    }

    let values::p2(ident) = vals.as_ref() else {
        return Err("expected enum ident, ident is not exist".into());
    };
    Ok(Box::new(Enum {
        ty,
        ident: ident.as_ref().0.content.into(),
        is_null: false,
    }))
}

fn parse_string_value(ty: TypeInfo, vals: &Box<values>) -> Result<Box<dyn Value>, error::Error> {
    match vals.as_ref() {
        values::p0(literal_vals) => {
            let literal_vals::p3(string_vals) = literal_vals.as_ref() else {
                return Err("expected literal vals when parse string value".into());
            };
            let raw = string_vals.as_ref().0.content;
            if raw.len() < 2 {
                return Err("raw string val is not long enough".into());
            }
            let raw = &raw[1..raw.len() - 1];
            Ok(Box::new(SString {
                ty,
                val: raw.into(),
                is_null: false,
            }) as _)
        }
        values::p3(_) => Ok(Box::new(SString {
            ty,
            val: "".into(),
            is_null: true,
        }) as _),
        _ => return Err("Invalid literal vals for string".into()),
    }
}

fn parse_bool_value(ty: TypeInfo, vals: &Box<values>) -> Result<Box<dyn Value>, error::Error> {
    let values::p0(literal_vals) = vals.as_ref() else {
        return Err("expected literal vals when parse bool value".into());
    };

    match literal_vals.as_ref() {
        literal_vals::p0(v) => match v.as_ref() {
            states::nodes::bool_literal::p0(_) | states::nodes::bool_literal::p1(_) => {
                Ok(Box::new(Bool { ty, val: true }) as _)
            }
            states::nodes::bool_literal::p2(_) | states::nodes::bool_literal::p3(_) => {
                Ok(Box::new(Bool { ty, val: false }) as _)
            }
        },
        literal_vals::p1(v) => {
            let val = get_number_value::<i32>(v)?;
            if val > 0 {
                Ok(Box::new(Bool { ty, val: true }) as _)
            } else {
                Ok(Box::new(Bool { ty, val: false }) as _)
            }
        }
        _ => return Err("Bool type need integer val or bool val".into()),
    }
}

fn parse_uint_value(ty: TypeInfo, vals: &Box<values>) -> Result<Box<dyn Value>, error::Error> {
    let values::p0(literal_vals) = vals.as_ref() else {
        return Err("expected literal vals when parse uint value".into());
    };

    let literal_vals::p1(integer_vals) = literal_vals.as_ref() else {
        return Err("expected integer vals when parse uint value".into());
    };
    let val = get_non_neg_integer_value(integer_vals)?;
    Ok(Box::new(UInt { ty, val }) as _)
}

fn parse_short_value(ty: TypeInfo, vals: &Box<values>) -> Result<Box<dyn Value>, error::Error> {
    let values::p0(literal_vals) = vals.as_ref() else {
        return Err("expected literal_vals".into());
    };

    let literal_vals::p1(integer_vals) = literal_vals.as_ref() else {
        return Err("expected integer_vals when parsing short value".into());
    };

    let val = get_number_value(integer_vals)?;
    Ok(Box::new(Short { ty, val }) as _)
}

fn parse_ushort_value(ty: TypeInfo, vals: &Box<values>) -> Result<Box<dyn Value>, error::Error> {
    let values::p0(literal_vals) = vals.as_ref() else {
        return Err("expected literal vals when parse ushort value".into());
    };

    let literal_vals::p1(integer_vals) = literal_vals.as_ref() else {
        return Err("expected integer_vals when parsing ushort value".into());
    };
    let val = get_non_neg_integer_value(integer_vals)?;
    Ok(Box::new(UShort { ty, val }) as _)
}

fn parse_byte_value(ty: TypeInfo, vals: &Box<values>) -> Result<Box<dyn Value>, error::Error> {
    let values::p0(literal_vals) = vals.as_ref() else {
        return Err("expected literal vals when parse byte value".into());
    };

    let literal_vals::p1(integer_vals) = literal_vals.as_ref() else {
        return Err("expected integer_vals when parsing byte value".into());
    };

    let val = get_non_neg_integer_value(integer_vals)?;
    Ok(Box::new(Byte { ty, val }) as _)
}

fn parse_sbyte_value(ty: TypeInfo, vals: &Box<values>) -> Result<Box<dyn Value>, error::Error> {
    let values::p0(literal_vals) = vals.as_ref() else {
        return Err("expected literal vals when parse sbyte value".into());
    };

    let literal_vals::p1(integer_vals) = literal_vals.as_ref() else {
        return Err("expected integer vals when parse sbyte value".into());
    };

    let val = get_number_value(integer_vals)?;
    Ok(Box::new(SByte { ty, val }) as _)
}

fn parse_list_value(
    raw: &Box<list_type>,
    ty: TypeInfo,
    vals: &Box<values>,
    ctx: &Context,
) -> Result<Box<dyn Value>, error::Error> {
    if let values::p3(_) = vals.as_ref() {
        return Ok(Box::new(List {
            ty,
            vals: Vec::new(),
            is_null: true,
        }) as _);
    }

    let list_type::p0(_, _, raw, _) = raw.as_ref();
    let values::p1(array_vals) = vals.as_ref() else {
        return Err("expected array_vals for List".into());
    };
    let mut rvals = Vec::new();

    match array_vals.as_ref() {
        states::nodes::array_vals::p0(_, _) => {
            return Ok(Box::new(List {
                ty,
                vals: Vec::new(),
                is_null: false,
            }) as _)
        }
        states::nodes::array_vals::p1(_, elements, _) => {
            parse_array_elements_value(raw, elements, &mut rvals, ctx)?
        }
        states::nodes::array_vals::p2(_, elements, _, _) => {
            parse_array_elements_value(raw, elements, &mut rvals, ctx)?
        }
    }
    Ok(Box::new(List {
        ty,
        vals: rvals,
        is_null: false,
    }) as _)
}

fn parse_array_value(
    raw: &Box<array_type>,
    ty: TypeInfo,
    vals: &Box<values>,
    ctx: &Context,
) -> Result<Box<dyn Value>, error::Error> {
    let (raw, nums) = match raw.as_ref() {
        array_type::p0(raw, _, _) => {
            if let values::p3(_) = vals.as_ref() {
                return Ok(Box::new(Array {
                    ty,
                    vals: Vec::new(),
                    is_null: true,
                }) as _);
            }
            (raw, None)
        }
        // Fixed Array
        array_type::p1(raw, _, _, _) => {
            if let values::p3(_) = vals.as_ref() {
                return Ok(Box::new(FixedArray {
                    ty,
                    vals: Vec::new(),
                    is_null: true,
                }) as _);
            }
            (raw, Some(()))
        }
    };

    let values::p1(array_vals) = vals.as_ref() else {
        return Err("expected array_vals for array".into());
    };
    let mut vals = Vec::new();

    match array_vals.as_ref() {
        states::nodes::array_vals::p0(_, _) => {
            if nums.is_none() {
                return Ok(Box::new(Array {
                    ty,
                    vals: Vec::new(),
                    is_null: false,
                }) as _);
            } else {
                return Ok(Box::new(FixedArray {
                    ty,
                    vals: Vec::new(),
                    is_null: false,
                }) as _);
            }
        }
        states::nodes::array_vals::p1(_, elements, _) => {
            parse_array_elements_value(raw, elements, &mut vals, ctx)?
        }
        states::nodes::array_vals::p2(_, elements, _, _) => {
            parse_array_elements_value(raw, elements, &mut vals, ctx)?
        }
    }

    if nums.is_none() {
        Ok(Box::new(Array {
            ty,
            vals,
            is_null: false,
        }) as _)
    } else {
        Ok(Box::new(FixedArray {
            ty,
            vals,
            is_null: false,
        }) as _)
    }
}

fn parse_valuetuple_value(
    raw: &Box<value_tuple_type>,
    ty: TypeInfo,
    vals: &Box<values>,
    ctx: &Context,
) -> Result<Box<dyn Value>, error::Error> {
    let raw = match raw.as_ref() {
        value_tuple_type::p0(_, _, ty, _) => ty,
        value_tuple_type::p1(_, _, ty, _, _) => ty,
    };
    let values::p1(valuetuple_vals) = vals.as_ref() else {
        return Err("expected array_vals for valuetuple".into());
    };
    let mut vals = Vec::new();

    match valuetuple_vals.as_ref() {
        states::nodes::array_vals::p0(_, _) => {
            return Err("ValueTuple<T> is not matched, caused by `the instances is empty`".into())
        }
        states::nodes::array_vals::p1(_, elements, _) => {
            parse_tuple_value_inner(raw, elements, &mut vals, ctx)?
        }
        states::nodes::array_vals::p2(_, elements, _, _) => {
            parse_tuple_value_inner(raw, elements, &mut vals, ctx)?
        }
    }
    Ok(Box::new(ValueTuple { ty, vals }) as _)
}

fn parse_tuple_value(
    raw: &Box<tuple_type>,
    ty: TypeInfo,
    vals: &Box<values>,
    ctx: &Context,
) -> Result<Box<dyn Value>, error::Error> {
    let raw = match raw.as_ref() {
        tuple_type::p0(_, _, ty, _) => ty,
        tuple_type::p1(_, _, ty, _, _) => ty,
    };
    let values::p1(tuple_vals) = vals.as_ref() else {
        return Err("expected array_vals for tuple".into());
    };
    let mut vals = Vec::new();

    match tuple_vals.as_ref() {
        states::nodes::array_vals::p0(_, _) => {
            return Err("Tuple<T> is not matched, caused by `the instances is empty`".into())
        }
        states::nodes::array_vals::p1(_, elements, _) => {
            parse_tuple_value_inner(raw, elements, &mut vals, ctx)?
        }
        states::nodes::array_vals::p2(_, elements, _, _) => {
            parse_tuple_value_inner(raw, elements, &mut vals, ctx)?
        }
    }
    Ok(Box::new(Tuple { ty, vals }) as _)
}

fn parse_tuple_value_inner(
    raw: &Box<tuple_type_elements>,
    elements: &Box<array_elements>,
    vals: &mut Vec<Box<dyn Value>>,
    ctx: &Context,
) -> Result<(), error::Error> {
    match raw.as_ref() {
        tuple_type_elements::p0(raw) => {
            let array_elements::p0(single) = elements.as_ref() else {
                return Err(
                    "the tuple has only one generic param, but the nums of args over `1`".into(),
                );
            };
            vals.push(get_value(raw, single, ctx)?);
        }
        tuple_type_elements::p1(prev, _, raw) => {
            let array_elements::p1(multi, _, single) = elements.as_ref() else {
                return Err(
                    "the tuple has multi generic params, but the nums of args are not enough"
                        .into(),
                );
            };
            parse_tuple_value_inner(prev, multi, vals, ctx)?;
            vals.push(get_value(raw, single, ctx)?);
        }
    }
    Ok(())
}

fn parse_custom_value(ty: TypeInfo, vals: &Box<values>) -> Result<Box<dyn Value>, error::Error> {
    if let values::p3(_) = vals.as_ref() {
        return Ok(Box::new(Custom {
            ty,
            args: Vec::new(),
            is_null: true,
        }) as _);
    }

    let values::p1(array_vals) = vals.as_ref() else {
        return Err("expected array_vals for custom type".into());
    };

    let mut args = Vec::new();
    match array_vals.as_ref() {
        states::nodes::array_vals::p0(_, _) => {
            return Ok(Box::new(Custom {
                ty,
                args: Vec::new(),
                is_null: false,
            }) as _)
        }
        states::nodes::array_vals::p1(_, elements, _)
        | states::nodes::array_vals::p2(_, elements, _, _) => {
            parse_custom_value_inner(elements, &mut args)?;
        }
    }
    Ok(Box::new(Custom {
        ty,
        args,
        is_null: false,
    }) as _)
}

fn get_raw_value(vals: &Box<values>) -> Result<String, error::Error> {
    match vals.as_ref() {
        values::p0(v) => get_raw_literal_value(v),
        values::p1(v) => match v.as_ref() {
            states::nodes::array_vals::p0(_, _) => Ok("{}".into()),
            states::nodes::array_vals::p1(_, vals, _)
            | states::nodes::array_vals::p2(_, vals, _, _) => {
                let mut arr = String::from("{");
                get_raw_array_value(vals, &mut arr)?;
                arr.push('}');
                Ok(arr)
            }
        },
        values::p2(v) => Ok(v.as_ref().0.content.into()),
        values::p3(_) => Ok("null".into()),
    }
}

fn get_raw_array_value(vals: &Box<array_elements>, arr: &mut String) -> Result<(), error::Error> {
    match vals.as_ref() {
        array_elements::p0(e) => arr.push_str(get_raw_value(e)?.as_str()),
        array_elements::p1(prev, _, last) => {
            get_raw_array_value(prev, arr)?;
            arr.push_str(", ");
            arr.push_str(get_raw_value(last)?.as_str());
        }
    }
    Ok(())
}

fn get_raw_literal_value(vals: &Box<literal_vals>) -> Result<String, error::Error> {
    match vals.as_ref() {
        literal_vals::p0(v) => match v.as_ref() {
            states::nodes::bool_literal::p0(_) => Ok("true".into()),
            states::nodes::bool_literal::p1(_) => Ok("true".into()),
            states::nodes::bool_literal::p2(_) => Ok("false".into()),
            states::nodes::bool_literal::p3(_) => Ok("false".into()),
        },
        literal_vals::p1(v) => match v.as_ref() {
            states::nodes::integer_literal::p0(_) => todo!(),
            states::nodes::integer_literal::p1(_, _) => todo!(),
            states::nodes::integer_literal::p2(_) => todo!(),
            states::nodes::integer_literal::p3(_, _) => todo!(),
            states::nodes::integer_literal::p4(v) => {
                let (_, minus) = v
                    .as_ref()
                    .0
                    .data
                    .get_custom()
                    .ok_or::<error::Error>("get custom cdata failed".into())?
                    .into_digits()
                    .ok_or::<error::Error>("".into())?;
                if minus {
                    Ok(format!("-{}", v.as_ref().0.content))
                } else {
                    Ok(format!("{}", v.as_ref().0.content))
                }
            }
            states::nodes::integer_literal::p5(_, _) => todo!(),
            states::nodes::integer_literal::p6(_) => todo!(),
            states::nodes::integer_literal::p7(_, _) => todo!(),
            states::nodes::integer_literal::p8(t, _, v) => {
                Ok(format!("{}.{}", t.as_ref().0.content, v.as_ref().0.content))
            }
        },
        literal_vals::p2(v) => match v.as_ref() {
            states::nodes::float_literal::p0(v) => Ok(v.as_ref().0.content.into()),
            states::nodes::float_literal::p1(v, _) => Ok(v.as_ref().0.content.into()),
            states::nodes::float_literal::p2(v1, _, v2) => Ok(format!(
                "{}.{}",
                v1.as_ref().0.content,
                v2.as_ref().0.content
            )),
            states::nodes::float_literal::p3(_, _, _, _) => todo!(),
            states::nodes::float_literal::p4(_, _, _) => todo!(),
            states::nodes::float_literal::p5(_, _, _, _) => todo!(),
        },
        literal_vals::p3(v) => Ok(v.as_ref().0.content.into()),
    }
}

fn parse_custom_value_inner(
    elements: &Box<array_elements>,
    vals: &mut Vec<String>,
) -> Result<(), error::Error> {
    match elements.as_ref() {
        array_elements::p0(v) => {
            vals.push(get_raw_value(v)?);
        }
        array_elements::p1(prev, _, v) => {
            parse_custom_value_inner(prev, vals)?;
            vals.push(get_raw_value(v)?);
        }
    }
    Ok(())
}

fn parse_array_elements_value(
    raw: &Box<value_type>,
    elements: &Box<array_elements>,
    vals: &mut Vec<Box<dyn Value>>,
    ctx: &Context,
) -> Result<(), error::Error> {
    match elements.as_ref() {
        array_elements::p0(v) => vals.push(get_value(raw, v, ctx)?),
        array_elements::p1(prev, _, v) => {
            parse_array_elements_value(raw, prev, vals, ctx)?;
            vals.push(get_value(raw, v, ctx)?);
        }
    }
    Ok(())
}

fn parse_int_value(ty: TypeInfo, vals: &Box<values>) -> Result<Box<dyn Value>, error::Error> {
    let values::p0(literal_vals) = vals.as_ref() else {
        return Err("expected literal vals when parse int value".into());
    };

    let literal_vals::p1(integer_vals) = literal_vals.as_ref() else {
        return Err("expected integer vals when parse int value".into());
    };

    let val = get_number_value(integer_vals)?;
    Ok(Box::new(Int { ty, val }) as _)
}

fn parse_float_value(ty: TypeInfo, vals: &Box<values>) -> Result<Box<dyn Value>, error::Error> {
    let values::p0(literal_vals) = vals.as_ref() else {
        return Err("expected literal vals when parse float value".into());
    };

    let val = match literal_vals.as_ref() {
        literal_vals::p1(integer_vals) => get_number_value(integer_vals)?,
        literal_vals::p2(float_vals) => match float_vals.as_ref() {
            states::nodes::float_literal::p0(v) => v.as_ref().0.content.parse()?,
            states::nodes::float_literal::p1(v, _) => v.as_ref().0.content.parse()?,
            states::nodes::float_literal::p2(v1, _, v2) => {
                format!("{}.{}", v1.as_ref().0.content, v2.as_ref().0.content).parse()?
            }
            states::nodes::float_literal::p3(v1, _, v2, _) => {
                format!("{}.{}", v1.as_ref().0.content, v2.as_ref().0.content).parse()?
            }
            states::nodes::float_literal::p4(_, _, _) => todo!(),
            states::nodes::float_literal::p5(_, _, _, _) => todo!(),
        },
        _ => return Err("Invalid literal vals for float".into()),
    };

    Ok(Box::new(Float { ty, val }) as _)
}

fn parse_double_value(ty: TypeInfo, vals: &Box<values>) -> Result<Box<dyn Value>, error::Error> {
    let values::p0(literal_vals) = vals.as_ref() else {
        return Err("expected literal vals when parse double value".into());
    };

    let val = match literal_vals.as_ref() {
        literal_vals::p1(integer_vals) => get_number_value(integer_vals)?,
        literal_vals::p2(float_vals) => match float_vals.as_ref() {
            states::nodes::float_literal::p0(v) => v.as_ref().0.content.parse()?,
            states::nodes::float_literal::p1(v, _) => v.as_ref().0.content.parse()?,
            states::nodes::float_literal::p2(v1, _, v2) => {
                format!("{}.{}", v1.as_ref().0.content, v2.as_ref().0.content).parse()?
            }
            states::nodes::float_literal::p3(v1, _, v2, _) => {
                format!("{}.{}", v1.as_ref().0.content, v2.as_ref().0.content).parse()?
            }
            states::nodes::float_literal::p4(_, _, _) => todo!(),
            states::nodes::float_literal::p5(_, _, _, _) => todo!(),
        },
        _ => return Err("Invalid literal vals for double".into()),
    };

    Ok(Box::new(Double { ty, val }) as _)
}

pub fn transfer_str_value(val: &str, ty: &TypeInfo) -> Result<String, error::Error> {
    let mut ret = String::new();
    match ty {
        TypeInfo::String | TypeInfo::LString => {
            if val.starts_with('\"') && val.ends_with('\"') {
                return Ok(format!("{}", val));
            } else {
                return Ok(format!("\"{}\"", val));
            }
        }
        TypeInfo::List(v) | TypeInfo::Array(v) | TypeInfo::FixedArray(v, _) => {
            if val == "" {
                return Ok("".into());
            }

            ret.push('{');
            for s in util::split(val)? {
                ret.push_str(transfer_str_value(s, v)?.as_str());
                ret.push(',');
            }
        }
        TypeInfo::Tuple(v) | TypeInfo::ValueTuple(v) => {
            if val == "" {
                return Ok("".into());
            }

            ret.push('{');
            let subvals = util::split(val)?;
            if subvals.len() != v.len() {
                return Err("The subvals of Tuple are not match with generic type".into());
            }
            unsafe {
                for i in 0..v.len() {
                    ret.push_str(
                        transfer_str_value(subvals.get_unchecked(i), v.get_unchecked(i))?.as_str(),
                    );
                    ret.push(',');
                }
            }
        }
        _ => {
            ret.push_str(val);
            return Ok(ret);
        }
    }

    ret.replace_range(ret.len() - 1.., "}");
    Ok(ret)
}
