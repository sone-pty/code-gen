use std::{collections::HashMap, hint::unreachable_unchecked, borrow::{Borrow, Cow}};

use dec2flt::raw_float::RawFloat;

use crate::{cursor::Cursor, ParseError};

use super::{Token, Tokenizer, Data};

pub const KIND_WHITESPACE_OR_COMMENT: u32 = 0;
pub const KIND_IDENT: u32 = 1;
pub const KIND_KEYWORD: u32 = 2;
pub const KIND_SYMBOL: u32 = 3;
pub const KIND_STRING: u32 = 4;
pub const KIND_INTEGER: u32 = 5;
pub const KIND_FLOAT: u32 = 6;
pub const KIND_CODE_BLOCK: u32 = 7;

pub struct Comment;

impl<'a, D, M> Tokenizer<'a, D, M> for Comment {
    fn tokenize(&self, cursor: &mut Cursor<'a>, _: &mut M) -> Option<Result<Token<'a, D>, ParseError>> {
        if cursor.first() == '/' {
            match cursor.second() {
                '/' => {
                    let offset = cursor.offset();
                    let row = cursor.row();
                    let col = cursor.col();
                    cursor.bump();
                    cursor.bump();
                    cursor.eat_while(|c| c != '\n');
                    Some(Ok(Token::from_cursor(cursor, KIND_WHITESPACE_OR_COMMENT, offset, Data::None, row, col)))
                }
                '*' => {
                    let offset = cursor.offset();
                    let row = cursor.row();
                    let col = cursor.col();
                    cursor.bump();
                    cursor.bump();
                    let mut depth = 1_usize;
                    while let Some(c) = cursor.bump() {
                        match c {
                            '/' if cursor.first() == '*' => {
                                cursor.bump();
                                depth += 1;
                            }
                            '*' if cursor.first() == '/' => {
                                cursor.bump();
                                depth -= 1;
                                if depth == 0 {
                                    return Some(Ok(Token::from_cursor(cursor, KIND_WHITESPACE_OR_COMMENT, offset, Data::None, row, col)));
                                }
                            }
                            _ => {}
                        }
                    }
    
                    Some(Err(ParseError {
                        file: cursor.file().clone(),
                        row: cursor.row(),
                        col: cursor.col(),
                        msg: "unterminated block comment".into(),
                    }))
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

pub fn is_whitespace(c: char) -> bool {
    matches!(
        c,
        // Usual ASCII suspects
        '\u{0009}'   // \t
        | '\u{000A}' // \n
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        | '\u{0020}' // space

        // NEXT LINE from latin1
        | '\u{0085}'

        // Bidi markers
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK

        // Dedicated whitespace characters from Unicode
        | '\u{2028}' // LINE SEPARATOR
        | '\u{2029}' // PARAGRAPH SEPARATOR
    )
}

pub struct Whitespace;

impl<'a, D, M> Tokenizer<'a, D, M> for Whitespace {
    fn tokenize(&self, cursor: &mut Cursor<'a>, _: &mut M) -> Option<Result<Token<'a, D>, ParseError>> {
        if is_whitespace(cursor.first()) {
            let offset = cursor.offset();
            let row = cursor.row();
            let col = cursor.col();
            cursor.bump();
            cursor.eat_while(is_whitespace);
            Some(Ok(Token::from_cursor(cursor, KIND_WHITESPACE_OR_COMMENT, offset, Data::None, row, col)))
        } else {
            None
        }
    }
}

pub struct Identifier;

impl<'a, D, M> Tokenizer<'a, D, M> for Identifier {
    fn tokenize(&self, cursor: &mut Cursor<'a>, _: &mut M) -> Option<Result<Token<'a, D>, ParseError>> {
        if is_id_start(cursor.first()) {
            let offset = cursor.offset();
            let row = cursor.row();
            let col = cursor.col();
            cursor.bump();
            cursor.eat_while(is_id_continue);
            let len = cursor.offset() - offset;
            let s = unsafe { cursor.sub_content(offset, len) };
            Some(Ok(Token {
                kind: KIND_IDENT,
                content: s,
                data: Data::String(s.into()),
                location: cursor.location_from(row, col),
            }))
        } else {
            None
        }
    }
}

pub fn is_id_start(c: char) -> bool {
    c == '_' || unicode_xid::UnicodeXID::is_xid_start(c) 
}

pub fn is_id_continue(c: char) -> bool {
    unicode_xid::UnicodeXID::is_xid_continue(c)
}


pub struct IdentifierKeyword<T> (pub T);

impl<'a, D, M, F: Fn(&str) -> Option<u32>> Tokenizer<'a, D, M> for IdentifierKeyword<F> {
    fn tokenize(&self, cursor: &mut Cursor<'a>, ctx: &mut M) -> Option<Result<Token<'a, D>, ParseError>> {
        if cursor.first() == 'r' && cursor.second() == '#' && is_id_start(cursor.nth(2)) {
            let offset = cursor.offset();
            let row = cursor.row();
            let col = cursor.col();
            cursor.bump();
            cursor.bump();
            cursor.bump();
            cursor.eat_while(is_id_continue);
            let len = cursor.offset() - offset;
            Some(Ok(Token {
                kind: KIND_IDENT,
                content: unsafe { cursor.sub_content(offset, len) },
                data: Data::String(unsafe {
                    cursor.sub_content(offset + 2, len - 2) 
                }.into()),
                location: cursor.location_from(row, col),
            }))
        } else {
            Identifier.tokenize(cursor, ctx)
                .map(|t| t.map(|mut t| {
                    if let Some(id) = (self.0)(unsafe { t.data.get_string().unwrap_unchecked().as_ref() }) {
                        t.kind = KIND_KEYWORD;
                        t.data = Data::Id(id);
                    }
                    t
                }))
        }
    }
}

pub fn identifier_keyword_with_sorted_array<'a>(keywords: &'a [(&str, u32)]) -> IdentifierKeyword<impl Fn(&str) -> Option<u32> + 'a> {
    IdentifierKeyword (move |s: &str| {
        match keywords.binary_search_by(|probe| probe.0.cmp(s)) {
            Ok(index) => unsafe {
                Some(keywords.get_unchecked(index).1)
            }
            Err(_) => None,
        }
    })
}

impl<'a, D, M, K: Borrow<str> + std::hash::Hash + Eq> Tokenizer<'a, D, M> for IdentifierKeyword<HashMap<K, u32>> {
    fn tokenize(&self, cursor: &mut Cursor<'a>, ctx: &mut M) -> Option<Result<Token<'a, D>, ParseError>> {
        if cursor.first() == 'r' && cursor.second() == '#' && is_id_start(cursor.nth(2)) {
            let offset = cursor.offset();
            let row = cursor.row();
            let col = cursor.col();
            cursor.bump();
            cursor.bump();
            cursor.bump();
            cursor.eat_while(is_id_continue);
            let len = cursor.offset() - offset;
            Some(Ok(Token {
                kind: KIND_IDENT,
                content: unsafe { cursor.sub_content(offset, len) },
                data: Data::String(unsafe {
                    cursor.sub_content(offset + 2, len - 2) 
                }.into()),
                location: cursor.location_from(row, col),
            }))
        } else {
            Identifier.tokenize(cursor, ctx).map(|t| t.map(|mut t| {
                if let Some(id) = self.0.get(unsafe {t.data.get_string().unwrap_unchecked().as_ref() }) {
                    t.kind = KIND_KEYWORD;
                    t.data = Data::Id(*id);
                }
                t
            }))
        }
    }
}

pub struct QuotedStringUnescaped<const QUOTE: char>;

impl<'a, D, M, const QUOTE: char> Tokenizer<'a, D, M> for QuotedStringUnescaped<QUOTE> {
    fn tokenize(&self, cursor: &mut Cursor<'a>, _: &mut M) -> Option<Result<Token<'a, D>, ParseError>> {
        if cursor.first() == QUOTE {
            let offset = cursor.offset();
            let row = cursor.row();
            let col = cursor.col();
            cursor.bump();
            while let Some(c) = cursor.bump() {
                if c == QUOTE {
                    let content;
                    let string_content;
                    let len = cursor.offset() - offset;
                    unsafe {
                        content = cursor.sub_content(offset, len);
                        string_content = std::str::from_utf8_unchecked(content.as_bytes().get_unchecked(1 .. len - 1));
                    }
                    return Some(Ok(Token {
                        kind: KIND_STRING,
                        content,
                        data: Data::String(string_content.into()),
                        location: cursor.location_from(row, col),
                    }));
                } else if c == '\\' {
                    let first = cursor.first();
                    if first == '\\' || first == QUOTE {
                        cursor.bump();
                    }
                }
            }
    
            Some(Err(ParseError {
                file: cursor.file().clone(),
                row: cursor.row(),
                col: cursor.col(),
                msg: "unterminated string".into(),
            }))
    
        } else {
            None
        }
    }
}



fn push_escaped(s: &mut Cow<str>, ch: char) {
    s.to_mut().push(ch);
}

fn push_unescaped(s: &mut Cow<str>, ch: char) {
    match s {
        Cow::Borrowed(t) => {
            let ptr = t.as_ptr();
            let len = t.len();
            *t = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len + ch.len_utf8())) };
        }
        Cow::Owned(t) => {
            t.push(ch);
        }
    }
}

pub struct QuotedString<const QUOTE: char>;

impl<const QUOTE: char> QuotedString<QUOTE> {
    pub fn parse_content<'a>(cursor: &mut Cursor<'a>) -> Result<Cow<'a, str>, ParseError> {
        let mut s = Cow::Borrowed(unsafe { cursor.sub_content(cursor.offset(), 0) });
    
        'main: loop {
            let char_row = cursor.row();
            let char_col = cursor.col();
            if let Some(c) = cursor.bump() {
                match c {
                    '\\' => {
                        let row = cursor.row();
                        let col = cursor.col();
                        if let Some(c) = cursor.bump() {
                            match c {
                                'r' => push_escaped(&mut s, '\r'),
                                'n' => push_escaped(&mut s, '\n'),
                                't' => push_escaped(&mut s, '\t'),
                                '\\' => push_escaped(&mut s, '\''),
                                '0' => push_escaped(&mut s, '\0'),
                                '"' => push_escaped(&mut s, '"'),
                                '\'' => push_escaped(&mut s, '\''),
                                'x' => {
                                    let mut code = 0_u32;
                                    for _ in 0..2 {
                                        let row = cursor.row();
                                        let col = cursor.col();
                                        if let Some(c) = cursor.bump() {
                                            match c {
                                                '0'..='9' => code = code * 16 + c as u32 - b'0' as u32,
                                                'A'..='F' => code = code * 16 + c as u32 - b'A' as u32 + 10,
                                                'a'..='f' => code = code * 16 + c as u32 - b'a' as u32 + 10,
                                                c if c == QUOTE => return Err(ParseError {
                                                    file: cursor.file().clone(),
                                                    row: char_row,
                                                    col: char_col,
                                                    msg: "numeric character escape is too short".into(),
                                                }),
                                                _ => return Err(ParseError {
                                                    file: cursor.file().clone(),
                                                    row, col,
                                                    msg: format!("invalid character in numeric character escape: `{}`", c.escape_default()).into(),
                                                }),
                                            }
                                            
                                        } else {
                                            break 'main;
                                        }
                                    }
                                    if code > 0x7F {
                                        return Err(ParseError {
                                            file: cursor.file().clone(),
                                            row: char_row,
                                            col: char_col,
                                            msg: "out of range hex escape".into(),
                                        });
                                    }
                                    push_escaped(&mut s, unsafe { char::from_u32_unchecked(code) });
                                }
                                'u' => {
                                    if cursor.bump() != Some('{') {
                                        return Err(ParseError {
                                            file: cursor.file().clone(),
                                            row: char_row,
                                            col: char_col,
                                            msg: "incorrect unicode escape sequence".into(),
                                        });
                                    }
                                    let offset = cursor.offset();
                                    loop {
                                        let row = cursor.row();
                                        let col = cursor.col();
                                        if let Some(c) = cursor.bump() {
                                            match c {
                                                '0'..='9' | 'A'..='F' | 'a'..='f' => {}
                                                '}' => {
                                                    let len = cursor.offset() - offset - 1;
                                                    if len > 6 {
                                                        return Err(ParseError {
                                                            file: cursor.file().clone(),
                                                            row: char_row,
                                                            col: char_col,
                                                            msg: "overlong unicode escape".into(),
                                                        });
                                                    }
                                                    let mut code = 0_u32;
                                                    for &ch in unsafe { cursor.sub_content(offset, len).as_bytes() } {
                                                        code = code * 16 + match ch {
                                                            b'0'..=b'9' => ch - b'0',
                                                            b'A'..=b'F' => ch - b'A' + 10,
                                                            b'a'..=b'f' => ch - b'a' + 10,
                                                            _ => unsafe { unreachable_unchecked() }
                                                        } as u32;
                                                    }
                                                    if code > 0x10FFFF {
                                                        return Err(ParseError {
                                                            file: cursor.file().clone(),
                                                            row: char_row,
                                                            col: char_col,
                                                            msg: "invalid unicode character escape".into(),
                                                        });
                                                    }
                                                    push_escaped(&mut s, unsafe { char::from_u32_unchecked(code) });
                                                    break;
                                                }
                                                c if c == QUOTE => return Err(ParseError {
                                                    file: cursor.file().clone(),
                                                    row: char_row,
                                                    col: char_col,
                                                    msg: "unterminated unicode escape".into(),
                                                }),
                                                _ => return Err(ParseError {
                                                    file: cursor.file().clone(),
                                                    row,
                                                    col,
                                                    msg: format!("invalid character in unicode escape: `{}`", c.escape_default()).into(),
                                                }),
                                            }
                                        } else {
                                            break 'main;
                                        }
                                    }
                                }
                                '.' | '+' => {}
                                _ => return Err(ParseError {
                                    file: cursor.file().clone(),
                                    row, col,
                                    msg: format!("unknown character escape: `{}`", c.escape_default()).into(),
                                }),
                            }
                        } else {
                            break 'main;
                        }
                    }
                    c if c == QUOTE => return Ok(s),
                    _ => push_unescaped(&mut s, c),
                }
            } else {
                break 'main;
            }
        }
    
        Err(ParseError {
            file: cursor.file().clone(),
            row: cursor.row(),
            col: cursor.col(),
            msg: "unterminated string".into(),
        })
    }
}

impl<'a, D, M, const QUOTE: char> Tokenizer<'a, D, M> for QuotedString<QUOTE> {
    fn tokenize(&self, cursor: &mut Cursor<'a>, _: &mut M) -> Option<Result<Token<'a, D>, ParseError>> {
        if cursor.first() != QUOTE {
            return None;
        }
        let offset = cursor.offset();
        let row = cursor.row();
        let col = cursor.col();
        cursor.bump();

        Some(Self::parse_content(cursor).map(|s| Token::from_cursor(cursor, KIND_STRING, offset, Data::String(s), row, col)))
    }
}

pub struct RawString;

impl<'a, D, M> Tokenizer<'a, D, M> for RawString {
    fn tokenize(&self, cursor: &mut Cursor<'a>, _: &mut M) -> Option<Result<Token<'a, D>, ParseError>> {
        if cursor.first() != 'r' {
            return None;
        }
        let offset = cursor.offset();
        let row = cursor.row();
        let col = cursor.col();
        let mut count = 0;
        loop {
            let idx = count + 1;
            let c = cursor.nth(idx);
            match c {
                '#' => {
                    count = idx;
                    continue;
                }
                '"' => {
                    break;
                }
                _ => return None,
            }
        }
        for _ in 0 .. count + 2 {
            cursor.bump();
        }
        let start = cursor.offset();
        let build = |cursor, len| Token::from_cursor(
            cursor, KIND_STRING, offset,
            Data::<D>::String(unsafe {
                cursor.sub_content(start, len).into()
            }), row, col,
        );
        if count == 0 {
            while let Some(c) = cursor.bump() {
                if c == '"' {
                    return Some(Ok(build(cursor, cursor.offset() - start - 1)))
                }
            }
        } else {
            let mut ending = None;
            while let Some(c) = cursor.bump() {
                if let Some(ref mut cnt) = ending {
                    match c {
                        '#' => {
                            *cnt += 1;
                            if *cnt == count {
                                return Some(Ok(build(cursor, cursor.offset() - start - count - 1)));
                            }
                        }
                        '"' => *cnt = 0,
                        _ => ending = None,
                    }
                } else if c == '"' {
                    ending = Some(0_usize);
                }
            }
        }
        Some(Err(ParseError {
            file: cursor.file().clone(),
            row: cursor.row(),
            col: cursor.col(),
            msg: "unterminated raw string".into(),
        }))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NumberBase {
    Binary,
    Octal,
    Hexadecimal,
    Decimal,
}

impl NumberBase {
    pub fn is_decimal(self) -> bool {
        matches!(self, Self::Decimal)
    }
}

pub struct NumberExponent<'a> {
    pub minus: bool,
    pub content: &'a str,
}

pub struct NumberDetail<'a> {
    pub offset: usize,
    pub row: usize,
    pub col: usize,

    pub base: NumberBase,
    pub integer: &'a str,
    pub fraction: Option<&'a str>,
    pub exponent: Option<NumberExponent<'a>>,
}

impl ToString for NumberDetail<'_> {
    fn to_string(&self) -> String {
        let mut bytes = Vec::new();
        for &b in self.integer.as_bytes() {
            if b != b'_' {
                bytes.push(b);
            }
        }
        if let Some(fraction) = self.fraction {
            bytes.push(b'.');
            for &b in fraction.as_bytes() {
                if b != b'_' {
                    bytes.push(b);
                }
            }
        }
        if let Some(ref exponent) = self.exponent {
            bytes.push(b'e');
            if exponent.minus {
                bytes.push(b'-');
            }
            for &b in exponent.content.as_bytes() {
                if b != b'_' {
                    bytes.push(b);
                }
            }
        }
        unsafe { String::from_utf8_unchecked(bytes) }
    }
}

impl<'a> NumberDetail<'a> {
    pub unsafe fn to_float_unchecked<F: RawFloat>(&self) -> F {
        let exponent: i16;
        if let Some(ref e) = self.exponent {
            if let Some(val) = parse_u64(10, e.content.as_bytes().iter().filter_map(|t| {
                let val = (*t).wrapping_sub(b'0');
                if val < 10 {
                    Some(val)
                } else {
                    None
                }
            })) {
                if e.minus {
                    if val > 0x8000 {
                        return F::from_u64_bits(0);
                    }
                    exponent = (-(val as i64)) as _;
                } else {
                    if val > 0x7FFF {
                        return F::INFINITY;
                    }
                    exponent = val as _;
                }
            } else {
                if e.minus {
                    return F::from_u64_bits(0);
                }
                return F::INFINITY;
            }
        } else {
            exponent = 0;
        }
        let input = dec2flt::input::Literal::new(self.integer, self.fraction.unwrap_or(""), exponent);
        dec2flt::dec2flt(&input)
    }

    pub fn integer_to_u64(&self) -> Option<u64> {
        match self.base {
            NumberBase::Binary => {
                parse_u64(2, self.integer.as_bytes().iter().filter_map(|t| {
                    let val = (*t).wrapping_sub(b'0');
                    if val < 2 {
                        Some(val)
                    } else {
                        None
                    }
                }))
            }
            NumberBase::Octal => {
                parse_u64(8, self.integer.as_bytes().iter().filter_map(|t| {
                    let val = (*t).wrapping_sub(b'0');
                    if val < 8 {
                        Some(val)
                    } else {
                        None
                    }
                }))
            }
            NumberBase::Decimal => {
                parse_u64(10, self.integer.as_bytes().iter().filter_map(|t| {
                    let val = (*t).wrapping_sub(b'0');
                    if val < 10 {
                        Some(val)
                    } else {
                        None
                    }
                }))
            }
            NumberBase::Hexadecimal => {
                parse_u64(16, self.integer.as_bytes().iter().filter_map(|t| {
                    match *t {
                        b'0'..=b'9' => Some(unsafe { (*t).unchecked_sub(b'0') }),
                        b'A'..=b'F' => Some(unsafe { (*t).unchecked_sub(b'A' - 10)}),
                        b'a'..=b'f' => Some(unsafe { (*t).unchecked_sub(b'a' - 10)}),
                        _ => None,
                    }
                }))
            }
        }
    }
}

pub fn number_detial<'a>(cursor: &mut Cursor<'a>) -> Option<Result<NumberDetail<'a>, ParseError>> {
    let first = cursor.first();
    if !first.is_ascii_digit() {
        return None;
    }

    let offset = cursor.offset();
    let row = cursor.row();
    let col = cursor.col();

    cursor.bump();

    let mut base = NumberBase::Decimal;
    let mut integer_offset = offset;

    if first == '0' {
        let has_digits = match cursor.first() {
            'b' => {
                base = NumberBase::Binary;
                cursor.bump();
                integer_offset = cursor.offset();
                let mut has_digits = false;
                loop {
                    let ch = cursor.first();
                    match ch {
                        '0' | '1' => has_digits = true,
                        '2'..='9' => return Some(Err(ParseError {
                            file: cursor.file().clone(),
                            row: cursor.row(),
                            col: cursor.col(),
                            msg: "invalid digit for binary literal".into(),
                        })),
                        '_' => (),
                        _ => break,
                    }
                    cursor.bump();
                }
                has_digits
            }
            'o' => {
                base = NumberBase::Octal;
                cursor.bump();
                integer_offset = cursor.offset();
                let mut has_digits = false;
                loop {
                    let ch = cursor.first();
                    match ch {
                        '0'..='7' => has_digits = true,
                        '8' | '9' => return Some(Err(ParseError {
                            file: cursor.file().clone(),
                            row: cursor.row(),
                            col: cursor.col(),
                            msg: "invalid digit for octal literal".into(),
                        })),
                        '_' => (),
                        _ => break,
                    }
                    cursor.bump();
                }
                has_digits
            }
            'x' => {
                base = NumberBase::Hexadecimal;
                cursor.bump();
                integer_offset = cursor.offset();
                let mut has_digits = false;
                loop {
                    let ch = cursor.first();
                    match ch {
                        '0'..='9' | 'A'..='F' | 'a'..='f' => has_digits = true,
                        '_' => (),
                        _ => break,
                    }
                    cursor.bump();
                }
                has_digits
            }
            '0'..='9' | '_' => {
                cursor.bump();
                cursor.eat_while(|ch| ch.is_ascii_digit() || ch == '_');
                true
            }
            _ => true,
        };
        if !has_digits {
            return Some(Err(ParseError {
                file: cursor.file().clone(),
                row, col,
                msg: "no valid digits found for number".into(),
            }));
        }
    } else {
        cursor.eat_while(|ch| ch.is_ascii_digit() || ch == '_');
    }

    let integer_len = cursor.offset() - integer_offset;

    let mut fraction = None;
    if cursor.first() == '.' {
        let ch = cursor.second();
        if ch != '.' && !is_id_start(ch) {
            cursor.bump();
            let fraction_offset = cursor.offset();
            if cursor.first().is_ascii_digit() {
                cursor.bump();
                cursor.eat_while(|ch| ch.is_ascii_digit() || ch == '_');
                let fraction_len = cursor.offset() - fraction_offset;
                fraction = Some(unsafe { cursor.sub_content(fraction_offset, fraction_len) });
            } else {
                fraction = Some(unsafe { cursor.sub_content(fraction_offset, 0) })
            }
        }
    }

    let mut exponent = None;

    match cursor.first() {
        'E' | 'e' => {
            cursor.bump();
            let minus = match cursor.first() {
                '+' => {
                    cursor.bump();
                    false
                }
                '-' => {
                    cursor.bump();
                    true
                }
                _ => false,
            };
            let exponent_offset = cursor.offset();
            let mut has_digits = false;
            loop {
                match cursor.first() {
                    '0'..='9' => {
                        has_digits = true;
                    }
                    '_' => (),
                    _ => break,
                }
                cursor.bump();
            }
            if !has_digits {
                return Some(Err(ParseError {
                    file: cursor.file().clone(),
                    row, col,
                    msg: "expected at least one digit in exponent".into(),
                }));
            }
            exponent = Some(NumberExponent {
                minus,
                content: unsafe { cursor.sub_content(exponent_offset, cursor.offset() - exponent_offset) },
            });
        }
        _ => (),
    }

    Some(Ok(NumberDetail {
        offset,
        row, col,

        base,
        integer: unsafe { cursor.sub_content(integer_offset, integer_len) },
        fraction,
        exponent,
    }))
}

pub fn parse_u64(radix: u8, digits: impl Iterator<Item = u8>) -> Option<u64> {
    let mut val = 0_u64;
    for digit in digits {
        val = val.checked_mul(radix as _)?.checked_add(digit as _)?;
    }
    Some(val)
}

pub struct Number;

impl<'a, D, M> Tokenizer<'a, D, M> for Number {
    fn tokenize(&self, cursor: &mut Cursor<'a>, _: &mut M) -> Option<Result<Token<'a, D>, ParseError>> {
        number_detial(cursor).map(|detail| {
            match detail {
                Ok(detail) => {
                    if detail.fraction.is_some() || detail.exponent.is_some() {
                        match detail.base {
                            NumberBase::Binary => return Err(ParseError {
                                file: cursor.file().clone(),
                                row: detail.row,
                                col: detail.col,
                                msg: "binary float literal is not supported".into(),
                            }),
                            NumberBase::Octal => return Err(ParseError {
                                file: cursor.file().clone(),
                                row: detail.row,
                                col: detail.col,
                                msg: "octal float literal is not supported".into(),
                            }),
                            NumberBase::Hexadecimal => return Err(ParseError {
                                file: cursor.file().clone(),
                                row: detail.row,
                                col: detail.col,
                                msg: "hexadecimal float literal is not supported".into(),
                            }),
                            _ => (),
                        }
    
                        return Ok(Token::from_cursor(cursor, KIND_FLOAT, detail.offset, Data::Float(unsafe { detail.to_float_unchecked() }), detail.row, detail.col))
                           
                    } else {
    
                        if let Some(val) = detail.integer_to_u64() {
                            return Ok(Token::from_cursor(cursor, KIND_INTEGER, detail.offset, Data::Integer(val), detail.row, detail.col));
                        }
    
                        return Err(ParseError {
                            file: cursor.file().clone(),
                            row: detail.row,
                            col: detail.col,
                            msg: "literal out of range".into(),
                        });
                    }
                }
                Err(e) => Err(e),
            }
            
        })
    }
}

pub struct Symbol<T> (pub T);

impl<'a, D, M, F: Fn(char) -> Option<u32>> Tokenizer<'a, D, M> for Symbol<F> {
    fn tokenize(&self, cursor: &mut Cursor<'a>, _: &mut M) -> Option<Result<Token<'a, D>, ParseError>> {
        (self.0)(cursor.first()).map(|id| {
            let offset = cursor.offset();
            let row = cursor.row();
            let col = cursor.col();
            cursor.bump();
            Ok(Token {
                kind: KIND_SYMBOL,
                content: unsafe { cursor.sub_content(offset, 1) },
                data: Data::Id(id),
                location: cursor.location_from(row, col),
            })
        })
    }
}

pub fn symbol_with_sorted_array(symbols: &[(char, u32)]) -> Symbol<impl Fn(char) -> Option<u32> + '_> {
    Symbol (move |ch| {
        match symbols.binary_search_by(|probe| probe.0.cmp(&ch)) {
            Ok(index) => unsafe { Some(symbols.get_unchecked(index).1) }
            Err(_) => None,
        }
    })
}

impl<'a, D, M> Tokenizer<'a, D, M> for Symbol<HashMap<char, u32>> {
    fn tokenize(&self, cursor: &mut Cursor<'a>, _: &mut M) -> Option<Result<Token<'a, D>, ParseError>> {
        self.0.get(&cursor.first()).map(|&id| {
            let offset = cursor.offset();
            let row = cursor.row();
            let col = cursor.col();
            cursor.bump();
            Ok(Token {
                kind: KIND_SYMBOL,
                content: unsafe { cursor.sub_content(offset, 1) },
                data: Data::Id(id),
                location: cursor.location_from(row, col),
            })
        })
    }
}

pub struct CodeBlock;

impl<'a, D, M> Tokenizer<'a, D, M> for CodeBlock {
    fn tokenize(&self, cursor: &mut Cursor<'a>, _: &mut M) -> Option<Result<Token<'a, D>, ParseError>> {
        if cursor.first() != '`' || cursor.second() != '`' || cursor.nth(2) != '`' {
            return None;
        }
        let offset = cursor.offset();
        let row = cursor.row();
        let col = cursor.col();
        cursor.bump();
        cursor.bump();
        cursor.bump();
        let mut count = 3;
        while cursor.first() == '`' {
            cursor.bump();
            count += 1;
        }
        let mut tc = 0;
        while let Some(t) = cursor.bump() {
            if t == '`' {
                tc += 1;
                if tc == count {
                    let len = cursor.offset() - offset;
                    return Some(Ok(Token {
                        kind: KIND_CODE_BLOCK,
                        content: unsafe { cursor.sub_content(offset, len) },
                        data: Data::CodeBlock(count, unsafe { cursor.sub_content(offset + count, len - count - count) }),
                        location: cursor.location_from(row, col),
                    }));
                }
            } else {
                tc = 0;
            }
        }

        Some(Err(ParseError {
            file: cursor.file().clone(),
            row: cursor.row(),
            col: cursor.col(),
            msg: "unterminated code block".into(),
        }))
    }
}