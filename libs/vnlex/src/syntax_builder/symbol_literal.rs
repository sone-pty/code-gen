use crate::{token::{tokenizers::QuotedString, Token, Tokenizer}, cursor::Cursor, ParseError};

pub fn symbol_literal<'a>(cursor: &mut Cursor<'a>, ctx: &mut ()) -> Option<Result<Token<'a, ()>, ParseError>> {
    QuotedString::<'\''>.tokenize(cursor, ctx).map(|t| t.map(|mut t| {
        t.kind = 102;
        t
    }))
}