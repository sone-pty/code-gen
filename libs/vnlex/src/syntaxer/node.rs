
use std::ops::ControlFlow;

use crate::token::Token;

use super::state::State;

pub trait Node<'a, T> {
    fn into_token(self: Box<Self>) -> Result<Box<(Token<'a, T>, bool)>, Box<dyn Node<'a, T> + 'a>>;
    fn destruct(self: Box<Self>, destructor: &mut dyn NodeDestructor<'a, T>) -> ControlFlow<()>;
}

impl<'a, T> dyn Node<'a, T> + 'a {
    pub unsafe fn downcast_unchecked<R: Sized>(self: Box<Self>) -> Box<R> {
        Box::from_raw(Box::into_raw(self) as *mut _)
    }
}

impl<'a, T> Node<'a, T> for (Token<'a, T>, bool) {
    fn into_token(self: Box<Self>) -> Result<Box<(Token<'a, T>, bool)>, Box<dyn Node<'a, T> + 'a>> {
        Ok(self)
    }
    fn destruct(self: Box<Self>, destructor: &mut dyn NodeDestructor<'a, T>) -> ControlFlow<()> {
        destructor.token(self)
    }
}

pub trait NonToken<'a, T>: Node<'a, T> {
    fn name(&self) -> &str;
    fn into_one(self: Box<Self>) -> Result<Box<dyn Node<'a, T> + 'a>, Box<dyn NonToken<'a, T> + 'a>>;
}

pub trait NodeDestructor<'a, T> {
    fn token(&mut self, token: Box<(Token<'a, T>, bool)>) -> ControlFlow<()>;
    fn non_token(&mut self, node: Box<dyn NonToken<'a, T> + 'a>) -> ControlFlow<()>;
}

pub trait RootNode<'a, T>: NonToken<'a, T> {
    fn entry_state<'r, F>(states: &'r [State<'r, F>]) -> &'r State<'r, F>;
}