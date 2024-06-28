//use core::str::next_code_point;
use std::{marker::PhantomData, collections::VecDeque, sync::Arc, path::PathBuf};

use crate::Location;



pub const EOF_CHAR: char = '\0';

pub struct Cursor<'a> {
    begin: *const u8,
    end: *const u8,
    ptr: *const u8,
    buf: VecDeque<(*const u8, char)>,
    file: Option<Arc<PathBuf>>,
    row: usize,
    col: usize,
    _marker: PhantomData<&'a str>,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str, row: usize, col: usize, file: Option<Arc<PathBuf>>) -> Self {
        let begin = input.as_ptr();
        let end = unsafe { begin.add(input.len()) };
        Cursor {
            begin,
            end,
            ptr: begin,
            buf: VecDeque::new(),
            file,
            row,
            col,
            _marker: PhantomData,
        }
    }

    pub fn first(&mut self) -> char {
        self.nth(0)
    }

    pub fn second(&mut self) -> char {
        self.nth(1)
    }

    pub fn nth(&mut self, n: usize) -> char {
        if let Some((_, c)) = self.buf.get(n) {
            *c
        } else {
            let mut count = n - self.buf.len();
            loop {
                let ptr = self.ptr;
                //if let Some(code) = unsafe { next_code_point(&mut Iter(self)) } {
                //    let c = unsafe { char::from_u32_unchecked(code) };
                if let Some(c) = unsafe { self.next_code_point() } {
                    self.buf.push_back((ptr, c));
                    if count == 0 {
                        break c;
                    }
                    count -= 1;
                } else {
                    break EOF_CHAR;
                }
            }
        }
    }

    pub fn is_eof(&self) -> bool {
        self.ptr == self.end && self.buf.is_empty()
    }

    pub fn bump(&mut self) -> Option<char> {
        if let Some((_, c)) = self.buf.pop_front() {
            self.update_row_col(c);
            Some(c)
        } else {
            //if let Some(code) = unsafe { next_code_point(&mut Iter(self)) } {
            //    let c = unsafe { char::from_u32_unchecked(code) };
            if let Some(c) = unsafe { self.next_code_point() } {
                self.update_row_col(c);
                Some(c)
            } else {
                None
            }
        }
    }

    pub fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while !self.is_eof() && predicate(self.first()) {
            self.bump();
        }
    }

    pub fn offset(&self) -> usize {
        let ptr = if let Some((ptr, _)) = self.buf.get(0) {
            *ptr
        } else {
            self.ptr
        };
        ptr as usize - self.begin as usize
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn file(&self) -> &Option<Arc<PathBuf>> {
        &self.file
    }

    pub fn location_from(&self, row: usize, col: usize) -> Location {
        Location::new(self.file.clone(), (row, col) .. (self.row, self.col))
    }

    pub fn content(&self) -> &'a str {
        unsafe {
            std::str::from_utf8_unchecked(
                std::slice::from_raw_parts(
                    self.begin,
                    self.end as usize - self.begin as usize,
                )
            )
        }
    }

    pub unsafe fn sub_content(&self, offset: usize, len: usize) -> &'a str {
        std::str::from_utf8_unchecked(
            std::slice::from_raw_parts(
                self.begin.add(offset),
                len,
            )
        )
    }

    fn update_row_col(&mut self, c: char) {
        if c != '\n' {
            self.col += 1;
        } else {
            self.col = 0;
            self.row += 1;
        }
    }

    unsafe fn next_code_point(&mut self) -> Option<char> {
        let mut chars = std::str::from_raw_parts(self.ptr, self.end.sub_ptr(self.ptr)).chars();
        let t = chars.next();
        self.ptr = chars.as_str().as_ptr();
        t
    }

}

// struct Iter<'r, 'a> (&'r mut Cursor<'a>);

// impl<'r, 'a> Iterator for Iter<'r, 'a> {
//     type Item = &'r u8;

//     fn next(&mut self) -> Option<Self::Item> {
//         let ptr = self.0.ptr;
//         if ptr != self.0.end {
//             unsafe {
//                 self.0.ptr = self.0.ptr.add(1); 
//                 Some(&*ptr)
//             }
//         } else {
//             None
//         }
//     }
// }
