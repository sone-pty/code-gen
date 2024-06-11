use super::cdata::CData;
use vnlex::{
    cursor::Cursor,
    token::{tokenizers::KIND_INTEGER, Data, Token, Tokenizer},
    ParseError,
};

pub struct Integer;

const KIND_BIN_INTEGER: u32 = 201;
const KIND_OCT_INTEGER: u32 = 202;
const KIND_HEX_INTEGER: u32 = 203;
const KIND_INTEGER_WITH_EXPONENT: u32 = 204;

impl<'a, M> Tokenizer<'a, CData<'a>, M> for Integer {
    fn tokenize(
        &self,
        cursor: &mut Cursor<'a>,
        _: &mut M,
    ) -> Option<Result<Token<'a, CData<'a>>, ParseError>> {
        let first = cursor.first();
        if !first.is_ascii_digit() {
            return None;
        }

        let offset = cursor.offset();
        let row = cursor.row();
        let col = cursor.col();

        cursor.bump();

        let mut integer_offset = offset;

        if first == '0' {
            match cursor.first() {
                'b' => {
                    cursor.bump();
                    integer_offset = cursor.offset();
                    let mut has_digits = false;
                    loop {
                        let ch = cursor.first();
                        match ch {
                            '0' | '1' => has_digits = true,
                            '2'..='9' => {
                                return Some(Err(ParseError::with_cursor(
                                    cursor,
                                    "invalid digit for binary literal",
                                )))
                            }
                            '_' => (),
                            _ => break,
                        }
                        cursor.bump();
                    }
                    if !has_digits {
                        return Some(Err(ParseError {
                            file: cursor.file().clone(),
                            row,
                            col,
                            msg: "no valid digits found for number".into(),
                        }));
                    }
                    let len = cursor.offset() - integer_offset;
                    let data = Data::Custom(CData::Digits(unsafe {
                        cursor.sub_content(integer_offset, len)
                    }));
                    return Some(Ok(Token::from_cursor(
                        cursor,
                        KIND_BIN_INTEGER,
                        offset,
                        data,
                        row,
                        col,
                    )));
                }
                'o' => {
                    cursor.bump();
                    integer_offset = cursor.offset();
                    let mut has_digits = false;
                    loop {
                        let ch = cursor.first();
                        match ch {
                            '0'..='7' => has_digits = true,
                            '8' | '9' => {
                                return Some(Err(ParseError::with_cursor(
                                    cursor,
                                    "invalid digit for octal literal",
                                )))
                            }
                            '_' => (),
                            _ => break,
                        }
                        cursor.bump();
                    }
                    if !has_digits {
                        return Some(Err(ParseError {
                            file: cursor.file().clone(),
                            row,
                            col,
                            msg: "no valid digits found for number".into(),
                        }));
                    }
                    let len = cursor.offset() - integer_offset;
                    let data = Data::Custom(CData::Digits(unsafe {
                        cursor.sub_content(integer_offset, len)
                    }));
                    return Some(Ok(Token::from_cursor(
                        cursor,
                        KIND_OCT_INTEGER,
                        offset,
                        data,
                        row,
                        col,
                    )));
                }
                'x' => {
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
                    if !has_digits {
                        return Some(Err(ParseError {
                            file: cursor.file().clone(),
                            row,
                            col,
                            msg: "no valid digits found for number".into(),
                        }));
                    }
                    let len = cursor.offset() - integer_offset;
                    let data = Data::Custom(CData::Digits(unsafe {
                        cursor.sub_content(integer_offset, len)
                    }));
                    return Some(Ok(Token::from_cursor(
                        cursor,
                        KIND_HEX_INTEGER,
                        offset,
                        data,
                        row,
                        col,
                    )));
                }
                '0'..='9' | '_' => {
                    cursor.bump();
                    cursor.eat_while(|ch| ch.is_ascii_digit() || ch == '_');
                }
                _ => (),
            }
        } else {
            cursor.eat_while(|ch| ch.is_ascii_digit() || ch == '_');
        }

        let integer_len = cursor.offset() - integer_offset;

        match cursor.first() {
            'E' | 'e' => {
                cursor.bump();
                let negative = match cursor.first() {
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
                        row,
                        col,
                        msg: "expected at least one digit in exponent".into(),
                    }));
                }
                let data = Data::Custom(CData::DecimalDigitsWithExponent(
                    unsafe { cursor.sub_content(integer_offset, integer_len) },
                    negative,
                    unsafe {
                        cursor.sub_content(exponent_offset, cursor.offset() - exponent_offset)
                    },
                ));
                return Some(Ok(Token::from_cursor(
                    cursor,
                    KIND_INTEGER_WITH_EXPONENT,
                    offset,
                    data,
                    row,
                    col,
                )));
            }
            _ => (),
        }

        let data = Data::Custom(CData::Digits(unsafe {
            cursor.sub_content(integer_offset, integer_len)
        }));

        Some(Ok(Token::from_cursor(
            cursor,
            KIND_INTEGER,
            offset,
            data,
            row,
            col,
        )))
    }
}
