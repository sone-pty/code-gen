use std::mem::MaybeUninit;


mod node;
mod unique;
mod shared;

pub use self::node::{ListNodeBlock, UniqueListNode, SharedListNode};

use self::node::ListNodePtr;

pub struct List<T: ?Sized, const N: usize = 0> {
    head: ListNodePtr<T>,
    tail: ListNodePtr<T>,
    len: usize,
}

impl<T: ?Sized, const N: usize> Default for List<T, N> {
    fn default() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

impl<T: ?Sized, const N: usize> List<T, N> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub unsafe fn front(&self) -> Option<&T> {
        if self.head.is_null() {
            None
        } else {
            Some(&*self.head.as_ptr())
        }
    }

    pub unsafe fn front_mut(&mut self) -> Option<&mut T> {
        if self.head.is_null() {
            None
        } else {
            Some(&mut *self.head.as_ptr())
        }
    }

    pub unsafe fn back(&self) -> Option<&T> {
        if self.tail.is_null() {
            None
        } else {
            Some(&*self.tail.as_ptr())
        }
    }

    pub unsafe fn back_mut(&mut self) -> Option<&T> {
        if self.tail.is_null() {
            None
        } else {
            Some(&mut *self.tail.as_ptr())
        }
    }
}