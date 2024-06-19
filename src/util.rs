use std::{
    path::Path,
    sync::atomic::{AtomicPtr, Ordering},
};

use xlsx_read::excel_file::ExcelFile;

use crate::{error::Error, table::TableEntity};

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
            v => {}
        }
    }
    Ok(entity)
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
