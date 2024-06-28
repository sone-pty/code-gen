use std::{sync::{LazyLock, Arc}, path::PathBuf};

use vnlex::{lexer::{Lexer, Builder}, syntaxer::Syntaxer, token::tokenizers, ParseError, cursor::Cursor, Location};

use crate::{Object, Array, states::nodes, Ident, Value, Null, Boolean, Integer, Float, String};

use super::states;


pub struct Parser {
    pub lexer: Lexer<(), ()>,
    pub syntaxer: Syntaxer<'static, states::ReductionType>,
}

impl Parser {
    fn new() -> Self {
        Self {
            lexer: Builder::whitespace()
                .append(tokenizers::Comment)
                .append(tokenizers::QuotedString::<'"'>)
                .append(tokenizers::RawString)
                .append(tokenizers::identifier_keyword_with_sorted_array(states::DEF_KEYWORDS))
                .append(tokenizers::Number)
                .append(tokenizers::symbol_with_sorted_array(states::DEF_SYMBOLS))
                .append(tokenizers::CodeBlock)
                .build(),
        
            syntaxer: Syntaxer::new(states::DEF_STATES),
        }
    }
}

pub static PARSER: LazyLock<Parser> = LazyLock::new(Parser::new);

type Result<T> = std::result::Result<T, ParseError>;

pub fn parse<'a>(content: &'a str, row: usize, col: usize, file: Option<Arc<PathBuf>>) -> Result<Object<'a>> {
    let parser = &*PARSER;
    let mut cursor = Cursor::new(content, row, col, file);
    let root = parser
        .syntaxer
        .parse_optional(parser.lexer.tokenizing(&mut cursor, &mut ()))
        .map_err(|t| t.into(&cursor))?;

    let mut obj = Object {
        base: Array {
            location: cursor.location_from(row, col),
            elements: Vec::new(),
        },
        ns: None,
        name: "".into(),
        attributes: crate::attributes::Attributes::new(),
    };

    if let Some(root) = root {
        parse_object_item_list(root, &mut obj)?;
    }
    Ok(obj)
}

fn parse_object(node: Box<nodes::object>) -> Result<Object> {
    Ok(match *node {
        nodes::object::p0(s, e) => Object {
            base: Array {
                location: Location::new(s.0.location.file.clone(), s.0.location.start() .. e.0.location.end()),
                elements: Vec::new(),
            },
            ns: None,
            name: "".into(),
            attributes: crate::attributes::Attributes::new(),
        },
        nodes::object::p1(s, n) => Object {
            base: Array {
                location: Location::new(s.0.location.file.clone(), s.0.location.start() .. n.0.location.end()),
                elements: Vec::new(),
            },
            ns: None,
            name: n.0.data.into_string().unwrap(),
            attributes: crate::attributes::Attributes::new(),
        },
        nodes::object::p2(s, n, _, e) => Object {
            base: Array {
                location: Location::new(s.0.location.file.clone(), s.0.location.start() .. e.0.location.end()),
                elements: Vec::new(),
            },
            ns: None,
            name: n.0.data.into_string().unwrap(),
            attributes: crate::attributes::Attributes::new(),
        },
        nodes::object::p3(s, node, e) => {
            let mut obj = Object {
                base: Array {
                    location: Location::new(s.0.location.file.clone(), s.0.location.start() .. e.0.location.end()),
                    elements: Vec::new(),
                },
                ns: None,
                name: "".into(),
                attributes: crate::attributes::Attributes::new(),
            };
            parse_object_item_list(node, &mut obj)?;
            obj
        }
        nodes::object::p4(s, n, _, node, e) => {
            let mut obj = Object {
                base: Array {
                    location: Location::new(s.0.location.file.clone(), s.0.location.start() .. e.0.location.end()),
                    elements: Vec::new(),
                },
                ns: None,
                name: n.0.data.into_string().unwrap(),
                attributes: crate::attributes::Attributes::new(),
            };
            parse_object_item_list(node, &mut obj)?;
            obj
        }
        nodes::object::p5(s, ns, _, name) => Object {
            base: Array {
                location: Location::new(s.0.location.file.clone(), s.0.location.start() .. name.0.location.end()),
                elements: Vec::new(),
            },
            ns: Some(ns.0.data.into_string().unwrap()),
            name: name.0.data.into_string().unwrap(),
            attributes: crate::attributes::Attributes::new(),
        },
        nodes::object::p6(s, ns, _, name, _, e) => Object {
            base: Array {
                location: Location::new(s.0.location.file.clone(), s.0.location.start() .. e.0.location.end()),
                elements: Vec::new(),
            },
            ns: Some(ns.0.data.into_string().unwrap()),
            name: name.0.data.into_string().unwrap(),
            attributes: crate::attributes::Attributes::new(),
        },
        nodes::object::p7(s, ns, _, name, _, node, e) => {
            let mut obj = Object {
                base: Array {
                    location: Location::new(s.0.location.file.clone(), s.0.location.start() .. e.0.location.end()),
                    elements: Vec::new(),
                },
                ns: Some(ns.0.data.into_string().unwrap()),
                name: name.0.data.into_string().unwrap(),
                attributes: crate::attributes::Attributes::new(),
            };
            parse_object_item_list(node, &mut obj)?;
            obj
        }
    })
}

fn parse_object_item_list<'a>(mut node: Box<nodes::object_item_list<'a>>, obj: &mut Object<'a>) -> Result<()> {
    let mut items = Vec::new();
    loop {
        match *node {
            nodes::object_item_list::p0(item) => {
                items.push(item);
                break;
            }
            nodes::object_item_list::p1(list, item) => {
                items.push(item);
                node = list;
            }
        }
    }
    for node in items.into_iter().rev() {
        parse_object_item(node, obj)?;
    }
    Ok(())
}

fn parse_object_item<'a>(node: Box<nodes::object_item<'a>>, obj: &mut Object<'a>) -> Result<()> {
    match *node {
        nodes::object_item::p0(token, _, node) |
        nodes::object_item::p2(token, _, node, _) => {
            let location = token.0.location;
            let name = token.0.data.into_string().unwrap();
            let key = Ident { location, value: name };
            use crate::attributes::Entry::*;
            match obj.attributes.entry(key) {
                Occupied(e) => Err(ParseError::with_location(&e.name().location, format!("duplicated attribute `{}`", e.name().value))),
                Vacant(e) => {
                    e.insert(parse_value(node)?);
                    Ok(())
                }
            }
        }
        nodes::object_item::p1(node) |
        nodes::object_item::p3(node, _) => {
            obj.base.elements.push(parse_value(node)?);
            Ok(())
        }
    }
}

pub fn parse_value(node: Box<nodes::value>) -> Result<Box<dyn Value + '_>> {
    Ok(match *node {
        nodes::value::p0(t) => Box::new(Null(t.0.location)),
        nodes::value::p1(t) => Box::new(Boolean {
            location: t.0.location,
            value: true,
        }),
        nodes::value::p2(t) => Box::new(Boolean {
            location: t.0.location,
            value: false,
        }),
        nodes::value::p3(t) => Box::new(Integer {
            location: t.0.location,
            minus: false,
            value: t.0.data.get_integer().unwrap(),
        }),
        nodes::value::p4(s, t) => Box::new(Integer {
            location: Location::new(s.0.location.file.clone(), s.0.location.start() .. t.0.location.end()),
            minus: true,
            value: t.0.data.get_integer().unwrap(),
        }),
        nodes::value::p5(t) => Box::new(Float {
            location: t.0.location,
            value: t.0.data.get_float().unwrap(),
        }),
        nodes::value::p6(s, t) => Box::new(Float {
            location: Location::new(s.0.location.file.clone(), s.0.location.start() .. t.0.location.end()),
            value: -t.0.data.get_float().unwrap(),
        }),
        nodes::value::p7(t) => Box::new(String {
            location: t.0.location,
            value: t.0.data.into_string().unwrap(),
        }),
        nodes::value::p8(t) => {
            let (cnt, content) = t.0.data.get_code_block().unwrap();
            let mut location = t.0.location;
            location.col += cnt;
            location.end_col -= cnt;
            Box::new(String {
                location,
                value: content.into(),
            })
        }
        nodes::value::p9(node) => Box::new(parse_object(node)?),
        nodes::value::p10(node) => Box::new(parse_array(node)?),
        nodes::value::p11(t) => Box::new(Ident {
            location: t.0.location,
            value: t.0.data.into_string().unwrap(),
        })
    })
}


fn parse_array(node: Box<nodes::array>) -> Result<Array> {
    Ok(match *node {
        nodes::array::p0(s, e) => Array {
            location: Location::new(s.0.location.file.clone(), s.0.location.start() .. e.0.location.end()),
            elements: Vec::new(),
        },
        nodes::array::p1(s, node, e) => Array {
            location: Location::new(s.0.location.file.clone(), s.0.location.start() .. e.0.location.end()),
            elements: parse_value_list(node)?,
        }
    })
}

fn parse_value_list(mut node: Box<nodes::value_list>) -> Result<Vec<Box<dyn Value + '_>>> {
    let mut ret = Vec::new();
    loop {
        match *node {
            nodes::value_list::p0(node) |
            nodes::value_list::p2(node, _)=> {
                ret.push(parse_value(node)?);
                break;
            }
            nodes::value_list::p1(list, value) |
            nodes::value_list::p3(list, value, _) => {
                ret.push(parse_value(value)?);
                node = list;
            }
        }
    }
    ret.reverse();
    Ok(ret)
}