use std::{
    path::Path,
    sync::atomic::{AtomicPtr, Ordering},
};

use xlsx_read::excel_file::ExcelFile;

use crate::{error::Error, preconfig::PRECONFIG, table::TableEntity};

pub fn conv_col_idx(mut n: usize) -> String {
    let mut result = String::new();
    while n > 0 {
        let rem = (n - 1) % 26;
        let letter = (b'A' + rem as u8) as char;
        result.push(letter);
        n = (n - 1) / 26;
    }
    result.chars().rev().collect()
}

pub fn load_execl_table<P: AsRef<Path>>(path: P, name: &str) -> Result<TableEntity, Error> {
    let mut excel = ExcelFile::load_from_path(path)?;
    let sheets = excel.parse_workbook()?;
    let mut entity = TableEntity::default();
    entity.name = name.into();

    for (flag, id) in sheets.into_iter() {
        let table = excel.parse_sheet(id)?;
        match flag.as_str() {
            "Template" => {
                entity.template = Some(table);
            }
            "GlobalConfig" => {
                entity.global = Some(table);
            }
            v if v.starts_with("t_") => {
                entity.enums.push(((&v[2..]).into(), table));
            }
            v => {
                if let Some(preconfig) = PRECONFIG.get(v) {
                    if preconfig.exist(v) {
                        entity.extras.push((v.into(), table));
                    }
                }
            }
        }
    }
    Ok(entity)
}

pub fn split(pat: &str) -> Vec<&str> {
    let pat_trim = pat.trim();
    let mut ret = Vec::new();

    if pat_trim.starts_with("{") && pat_trim.ends_with("}") {
        let mut brackets = Stack::new();
        let mut begin = 1;
        let mut idx = 0;

        for v in pat_trim.chars() {
            match v {
                '{' => {
                    if idx != 0 {
                        if brackets.is_empty() {
                            begin = idx;
                        }
                        brackets.push(v);
                    }
                }
                '}' => {
                    if idx == pat_trim.len() - 1 {
                        if begin < idx {
                            ret.push(&pat_trim[begin..idx]);
                        } else {
                            ret.push("");
                        }
                    } else {
                        let _ = brackets.pop();
                    }
                }
                ',' => {
                    if brackets.is_empty() {
                        ret.push(&pat_trim[begin..idx]);
                        begin = idx + 1;
                    }
                }
                _ => {}
            }
            idx += v.len_utf8();
        }
    }
    ret
}

struct Node<T> {
    data: T,
    next: AtomicPtr<Node<T>>,
}

pub struct AtomicLinkedList<T> {
    head: AtomicPtr<Node<T>>,
}

#[allow(dead_code)]
impl<T> AtomicLinkedList<T> {
    pub fn new() -> Self {
        AtomicLinkedList {
            head: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    pub fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(Node {
            data,
            next: AtomicPtr::new(std::ptr::null_mut()),
        }));

        loop {
            let head = self.head.load(Ordering::Acquire);
            unsafe {
                (*new_node).next.store(head, Ordering::Relaxed);
            }
            if self
                .head
                .compare_exchange(head, new_node, Ordering::Release, Ordering::Relaxed)
                .is_ok_and(|prev| prev == head)
            {
                break;
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);

            if head.is_null() {
                return None;
            }

            let next = unsafe { (*head).next.load(Ordering::Relaxed) };

            if self
                .head
                .compare_exchange(head, next, Ordering::Release, Ordering::Relaxed)
                .is_ok_and(|prev| prev == head)
            {
                unsafe {
                    let node = Box::from_raw(head);
                    return Some(node.data);
                }
            }
        }
    }

    pub unsafe fn into_unsafe_vector(self) -> Vec<T> {
        let mut ret = Vec::new();
        let mut p = self.head.load(Ordering::Relaxed);

        while !p.is_null() {
            let boxed_node = Box::from_raw(p);
            ret.push(boxed_node.data);
            p = boxed_node.next.load(Ordering::Relaxed);
        }
        ret
    }
}

pub struct Stack<T> {
    head: Link<T>,
}

type Link<T> = Option<Box<STNode<T>>>;

struct STNode<T> {
    elem: T,
    next: Link<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self { head: None }
    }

    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(STNode {
            elem,
            next: self.head.take(),
        });
        self.head = Some(new_node);
    }

    pub fn pop(&mut self) -> Result<T, &str> {
        match self.head.take() {
            None => Err("Stack is empty"),
            Some(node) => {
                self.head = node.next;
                Ok(node.elem)
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    #[allow(dead_code)]
    pub fn peek(&self) -> Option<&T> {
        match self.head.as_ref() {
            None => None,
            Some(node) => Some(&node.elem),
        }
    }

    #[allow(dead_code)]
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        match self.head.as_mut() {
            None => None,
            Some(node) => Some(&mut node.elem),
        }
    }

    #[allow(dead_code)]
    pub fn into_iter_for_stack(self) -> IntoIter<T> {
        IntoIter(self)
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }

    #[allow(dead_code)]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            next: self.head.as_deref_mut(),
        }
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.next.take();
        }
    }
}

pub struct IntoIter<T>(Stack<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop().ok()
    }
}

pub struct Iter<'a, T> {
    next: Option<&'a STNode<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut STNode<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();
            &mut node.elem
        })
    }
}
