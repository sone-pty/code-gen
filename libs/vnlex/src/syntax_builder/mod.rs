use std::{sync::{LazyLock, Arc}, borrow::Cow, path::{Path, PathBuf}};



use crate::{lexer::{Lexer, self}, syntaxer::Syntaxer, token::tokenizers, ParseError, cursor::Cursor};

use self::{syntax_desc::SyntaxDesc, syntax_info::SyntaxInfo};


mod syntax_info;
pub mod syntax_desc;
mod states;
mod keyword_literal;
mod symbol_literal;
mod node_ins;



struct Builder {
    lexer: Lexer<(), ()>,
    syntaxer: Syntaxer<'static, states::ReductionType>,
}

impl Builder {
    fn new() -> Self {
        Self {
            lexer: lexer::Builder::whitespace()
                .append(tokenizers::Comment)
                .append(tokenizers::identifier_keyword_with_sorted_array(states::DEF_KEYWORDS))
                .append(tokenizers::Number)
                .append(tokenizers::symbol_with_sorted_array(states::DEF_SYMBOLS))
                .append(keyword_literal::keyword_literal)
                .append(symbol_literal::symbol_literal)
                .build(),
            syntaxer: Syntaxer::new(states::DEF_STATES),
        }
    }
}

static BUILDER: LazyLock<Builder> = LazyLock::new(Builder::new);


pub fn parse<'a, 'f, CT: Into<Cow<'a, str>>>(content: &'a str, file: Option<Arc<PathBuf>>, mod_root: &Path, custom_type: CT) -> Result<SyntaxDesc, ParseError> {
    let mut cursor = Cursor::new(content, 0, 0, file);
    let info = SyntaxInfo::parse(&mut cursor, mod_root)?;
    SyntaxDesc::new(info, custom_type.into().into_owned()).ok_or_else(|| ParseError::with_cursor(
        &cursor,
        "fatal error(s) occurred",
    ))
}

fn parse_root<'a>(cursor: &mut Cursor<'a>) -> Result<Option<Box<states::nodes::script<'a>>>, ParseError> {
    let builder = &*BUILDER;
    builder.syntaxer.parse_optional(builder.lexer.tokenizing(cursor, &mut ())).map_err(|e| {
        match e {
            crate::syntaxer::Error::LexerError(e) => e,
            crate::syntaxer::Error::UnexpectedToken(token) => {
                ParseError::with_location(
                    &token.0.location,
                format!("unexpected `{}`", token.0.content.escape_default()),
                )
            },
            crate::syntaxer::Error::UnexpectedEOF => ParseError::with_cursor(
                cursor,
                "unexpected EOF",
            ),
        }
    })
}