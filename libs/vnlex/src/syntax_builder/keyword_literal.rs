
use crate::{cursor::Cursor, token::{Token, tokenizers::{is_id_start, is_id_continue}, Data}, ParseError};

pub fn keyword_literal<'a>(cursor: &mut Cursor<'a>, _: &mut ()) -> Option<Result<Token<'a, ()>, ParseError>> {
    if cursor.first() != '"' {
        return None;
    }

    let offset = cursor.offset();
    let row = cursor.row();
    let col = cursor.col();

    cursor.bump();

    if let Some(ch) = cursor.bump() {
        if !is_id_start(ch) {
            return Some(Err(ParseError::with_cursor(
                cursor,
                format!("invalid keyword start character `{}`", ch.escape_default()),
            )));
        }
        while let Some(ch) = cursor.bump() {
            if ch == '"' {
                let len = cursor.offset() - offset;
                return Some(Ok(Token::from_cursor(
                    cursor, 101, offset,
                    Data::String(unsafe {cursor.sub_content(offset + 1, len - 2)}.into()),
                    row, col,
                )));
            } else if !is_id_continue(ch) {
                return Some(Err(ParseError::with_cursor(
                    cursor,
                    format!("invalid keyword character `{}`", ch.escape_default()),
                )));
            }
        }
    }

    Some(Err(ParseError::with_cursor(cursor, "unterminated keyword")))
}