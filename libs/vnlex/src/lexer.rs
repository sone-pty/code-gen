use std::{sync::Arc, marker::{Unsize, PhantomData}, ops::Deref};

use crate::{token::{Tokenizer, Token, tokenizers}, cursor::Cursor, ParseError};



pub struct RawLexer<T: ?Sized> (Vec<Arc<T>>);

pub struct RawBuilder<T: ?Sized> (Vec<Arc<T>>);

impl<T: ?Sized> RawBuilder<T> {
    pub fn new() -> RawBuilder<T> {
        RawBuilder(Vec::new())
    }

    pub fn whitespace() -> RawBuilder<T>
    where
        tokenizers::Whitespace: Unsize<T> + 'static
    {
        Self::new().append(tokenizers::Whitespace)
    }

    pub fn append(mut self, tokenizer: impl Unsize<T> + 'static) -> Self {
        let tokenizer = Arc::new(tokenizer);
        self.0.push(tokenizer);
        self
    }

    pub fn build(self) -> RawLexer<T> {
        RawLexer (self.0)
    }
}

impl<T: ?Sized> RawLexer<T> {
    pub fn tokenize<'a, D, M>(&self, cursor: &mut Cursor<'a>, ctx: &mut M) -> Option<Result<Token<'a, D>, ParseError>>
    where
        T: Tokenizer<'a, D, M>,
    {
        if cursor.is_eof() {
            None
        } else {
            for tokenizer in self.0.iter() {
                if let Some(r) = tokenizer.tokenize(cursor, ctx) {
                    return Some(r);
                }
            }
            let ch = cursor.first();
            Some(Err(ParseError::with_cursor(
                cursor,
                format!("unexpected character `{}`", ch.escape_default()),
            )))
        }
    }

    pub fn tokenizing<'r, 'a, D, M>(&'r self, cursor: &'r mut Cursor<'a>, ctx: &'r mut M) -> Tokenizing<'r, 'a, T, M, D>
    where
        T: Tokenizer<'a, D, M>,
    {
        Tokenizing { 
            lexer: self,
            cursor, ctx,
            phantom: PhantomData,
        }
    }

}


pub struct Lexer<T, M> (RawLexer<dyn for<'a, 'f> Tokenizer<'a, T, M> + Send + Sync>);

impl<T, M> Deref for Lexer<T, M> {
    type Target = RawLexer<dyn for<'a, 'f> Tokenizer<'a, T, M> + Send + Sync>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Builder<T, M> (RawBuilder<dyn for<'a> Tokenizer<'a, T, M> + Send + Sync>);

impl<T, M> Builder<T, M> {
    pub fn new() -> Self {
        Builder (RawBuilder::new())
    }

    pub fn whitespace() -> Self
    where
        T: 'static,
    {
        Builder (RawBuilder::whitespace())
    }

    pub fn append(self, tokenizer: impl for<'a> Tokenizer<'a, T, M> + Send + Sync + 'static) -> Self {
        Builder (self.0.append(tokenizer))
    }

    pub fn build(self) -> Lexer<T, M> {
        Lexer (self.0.build())
    }
}


pub struct Tokenizing<'r, 'a, T: ?Sized, M, D> {
    lexer: &'r RawLexer<T>,
    cursor: &'r mut Cursor<'a>,
    ctx: &'r mut M,
    phantom: PhantomData<D>
} 

impl<'r, 'a, T, D, M> Iterator for Tokenizing<'r, 'a, T, M, D>
where
    T: ?Sized + Tokenizer<'a, D, M>,
{
    type Item = Result<Token<'a, D>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.tokenize(self.cursor, self.ctx)    
    }
}