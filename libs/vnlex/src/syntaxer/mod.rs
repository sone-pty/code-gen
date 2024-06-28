
use std::{fmt, marker::PhantomData};

use crate::{ParseError, token::{Token, tokenizers::KIND_WHITESPACE_OR_COMMENT}, cursor::Cursor};

pub use self::node::{Node, NonToken, RootNode, NodeDestructor};
use self::state::Reduction;


pub mod state;
mod node;

pub enum Error<'a, T> {
    LexerError(ParseError),
    UnexpectedToken(Box<(Token<'a, T>, bool)>),
    UnexpectedEOF,
}

impl<T> fmt::Display for Error<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::LexerError(e) => e.fmt(f),
            Error::UnexpectedToken(token) => {
                write!(f, "{}: unexpected `{}`", token.0.location, token.0.content.escape_default())
            }
            Error::UnexpectedEOF => write!(f, "unexpected EOF"),
        }
    }
}

impl<'a, T> Error<'a, T> {
    pub fn into(self, cursor: &Cursor<'a>) -> ParseError {
        match self {
            Error::LexerError(e) => e,
            Error::UnexpectedToken(token) => ParseError::with_location(
                &token.0.location,
                format!("unexpected `{}`", token.0.content.escape_default()),
            ),
            Error::UnexpectedEOF => ParseError::with_cursor( 
                cursor,
                "unexpected EOF",
            ),
        }
    }
}

pub struct Syntaxer<'s, F> (&'s [state::State<'s, F>]);

impl<'s, F> Syntaxer<'s, F> {
    pub fn new(states: &'s [state::State<'s, F>]) -> Self {
        Self (states)
    }
}

impl<'s, F> Syntaxer<'s, Reduction<F>> {
    pub fn parse<'a, I, D, N>(&self, mut tokens: I) -> Result<Box<N>, Error<'a, D>>
    where
        F: Fn(&mut Vec<Box<dyn Node<'a, D> + 'a>>, &mut Vec<Box<(Token<'a, D>, bool)>>) -> Box<dyn Node<'a, D> + 'a>,
        I: Iterator<Item = Result<Token<'a, D>, ParseError>>,
        D: 'a,
        N: RootNode<'a, D>,
    {
        let mut ctx = Context::new(self.0, N::entry_state(self.0));
        let mut followed = true;
        while let Some(token) = tokens.next() {
            let token = match token {
                Ok(t) => t,
                Err(e) => return Err(Error::LexerError(e)),
            };
            if token.kind == KIND_WHITESPACE_OR_COMMENT {
                followed = false;
                continue;
            }
            if let Some(token) = ctx.input(Box::new((token, followed))) {
                return Err(Error::UnexpectedToken(token));
            }
            followed = true;
        }
        ctx.finish().map(|t| unsafe { t.downcast_unchecked() })
    }

    pub fn parse_optional<'a, I, D, N>(&self, mut tokens: I) -> Result<Option<Box<N>>, Error<'a, D>>
    where
        F: Fn(&mut Vec<Box<dyn Node<'a, D> + 'a>>, &mut Vec<Box<(Token<'a, D>, bool)>>) -> Box<dyn Node<'a, D> + 'a>,
        I: Iterator<Item = Result<Token<'a, D>, ParseError>>,
        D: 'a,
        N: RootNode<'a, D>,
    {
        let mut ctx = Context::new(self.0, N::entry_state(self.0));
        let mut followed = true;
        while let Some(token) = tokens.next() {
            let token = match token {
                Ok(t) => t,
                Err(e) => return Err(Error::LexerError(e)),
            };
            if token.kind == KIND_WHITESPACE_OR_COMMENT {
                followed = false;
                continue;
            }
            if let Some(token) = ctx.input(Box::new((token, followed))) {
                return Err(Error::UnexpectedToken(token));
            }
            followed = true;
        }
        if ctx.nodes.is_empty() {
            return Ok(None);
        }
        ctx.finish().map(|t| unsafe { Some(t.downcast_unchecked()) })
    }

    pub fn parsing<'a, D, N>(&self) -> Parsing<'s, 'a, Reduction<F>, D, N>
    where
        F: Fn(&mut Vec<Box<dyn Node<'a, D> + 'a>>, &mut Vec<Box<(Token<'a, D>, bool)>>) -> Box<dyn Node<'a, D> + 'a>,
        N: RootNode<'a, D>,
        D: 'a,
    {
        Parsing { ctx: Context::new(self.0, N::entry_state(self.0)), _marker: PhantomData }
    }
}

struct Context<'s, 'a, R, D> {
    nodes: Vec<Box<dyn Node<'a, D> + 'a>>,
    stack: Vec<&'s state::State<'s, R>>,
    states: &'s [state::State<'s, R>],
    tokens: Vec<Box<(Token<'a, D>, bool)>>,
}

impl<'s, 'a, F, D> Context<'s, 'a, Reduction<F>, D>
where 
    F: Fn(&mut Vec<Box<dyn Node<'a, D> + 'a>>, &mut Vec<Box<(Token<'a, D>, bool)>>) -> Box<dyn Node<'a, D> + 'a>,
    D: 'a,
{
    fn new(states: &'s [state::State<'s, Reduction<F>>], entry: &'s state::State<'s, Reduction<F>>) -> Self {
        Self { 
            nodes: Vec::with_capacity(32),
            stack: {
                let mut stack = Vec::with_capacity(32);
                stack.push(entry);
                stack
            },
            states,
            tokens: Default::default(),
        }
    }

    fn input(&mut self, token: Box<(Token<'a, D>, bool)>) -> Option<Box<(Token<'a, D>, bool)>> {
        self.tokens.push(token);

        loop {
            if !self.push() {
                if !self.reduce() {
                    return self.tokens.pop();
                }
                continue;
            }
            if self.tokens.is_empty() {
                break None;
            }
        }
    }

    fn reduce(&mut self) -> bool {
        let mut state = unsafe { self.last_state() };
        if let Some(reduction) = state.reduction {
            let off = self.stack.len() - reduction.state_count;
            state = unsafe { self.stack.get_unchecked(off - 1) };
            if let Ok(index) = state.node_jumps.binary_search_by(|probe| probe.id.cmp(&reduction.node_id)) {
                let target = unsafe { state.node_jumps.get_unchecked(index).target };
                self.stack.truncate(off);
                let node = (reduction.production)(&mut self.nodes, &mut self.tokens);
                self.jump(target, node);
                return true;
            }
        }
        false
    }

    fn finish(&mut self) -> Result<Box<dyn Node<'a, D> + 'a>, Error<'a, D>> {
        loop {
            let mut state = unsafe { self.last_state() };
            if let Some(reduction) = state.reduction {
                if reduction.node_id == 0 {
                    return Ok(self.nodes.pop().unwrap());
                }
                let off = self.stack.len() - reduction.state_count;
                state = unsafe { self.stack.get_unchecked(off - 1) };
                if let Ok(index) = state.node_jumps.binary_search_by(|probe| probe.id.cmp(&reduction.node_id)) {
                    let target = unsafe { state.node_jumps.get_unchecked(index).target };
                    self.stack.truncate(off);
                    let node = (reduction.production)(&mut self.nodes, &mut self.tokens);
                    self.jump(target, node);
                } else {
                    unreachable!()
                }
            } else {
                return Err(Error::UnexpectedEOF);
            }
            if !self.tokens.is_empty() {
                loop {
                    if !self.push() {
                        if !self.reduce() {
                            return Err(Error::UnexpectedToken(self.tokens.pop().unwrap()));
                        }
                        continue;
                    }
                    if self.tokens.is_empty() {
                        break;
                    }
                }
            }
        }
    }

    fn push(&mut self) -> bool {
        let token = unsafe { self.tokens.last().unwrap_unchecked() };
        let state = unsafe { self.last_state() };
        let kind = token.0.kind;
        let id = token.0.data.get_id();
        if let Ok(index) = state.token_jumps.binary_search_by(|probe| {
            probe.kind.cmp(&kind).then_with(|| probe.id.cmp(&id))
        }) {
            let jump = unsafe { state.token_jumps.get_unchecked(index) };
            if token.1 {
                if let Some(target) = jump.followed_target {
                    let token = unsafe { self.tokens.pop().unwrap_unchecked() };
                    self.jump(target, token);
                    return true;
                }
            }
            if let Some(target) = jump.not_followed_target {
                let token = unsafe { self.tokens.pop().unwrap_unchecked() };
                self.jump(target, token);
                return true;
            }
        }

        false
    }

    fn jump(&mut self, target: usize, node: Box<dyn Node<'a, D> + 'a>) {
        self.stack.push(unsafe { self.states.get_unchecked(target) });
        self.nodes.push(node);
    }

    unsafe fn last_state(&self) -> &'s state::State<'s, Reduction<F>> {
        self.stack.last().unwrap_unchecked()
    }
}

pub enum ParsingError<'a, D> {
    NoInput,
    UnexpectedEOF,
    UnexpectedToken(Box<(Token<'a, D>, bool)>),
}

pub struct Parsing<'s, 'a, R, D, N> {
    ctx: Context<'s, 'a, R, D>,
    _marker: PhantomData<N>,
}

impl<'s, 'a, F, D, N> Parsing<'s, 'a, Reduction<F>, D, N>
where 
    F: Fn(&mut Vec<Box<dyn Node<'a, D> + 'a>>, &mut Vec<Box<(Token<'a, D>, bool)>>) -> Box<dyn Node<'a, D> + 'a>,
    D: 'a,
    N: RootNode<'a, D>,
{
    pub fn input(&mut self, token: Box<(Token<'a, D>, bool)>) -> Option<Box<(Token<'a, D>, bool)>> {
        self.ctx.input(token)
    }

    pub fn finish(mut self) -> Result<Box<N>, ParsingError<'a, D>> {
        if self.ctx.nodes.is_empty() {
            Err(ParsingError::NoInput)
        } else {
            match self.ctx.finish() {
                Ok(t) => Ok(unsafe { t.downcast_unchecked() }),
                Err(e) => Err(match e {
                    Error::UnexpectedEOF => ParsingError::UnexpectedEOF,
                    Error::UnexpectedToken(t) => ParsingError::UnexpectedToken(t),
                    _ => unreachable!()
                })
            }
        }
    }
}