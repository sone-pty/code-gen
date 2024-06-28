use std::{collections::HashMap, fmt, iter::FusedIterator, marker::PhantomData, mem::{forget, ManuallyDrop}};

use crate::{Ident, Value};

pub struct Attributes<'a> {
    indices: HashMap<&'static str, Box<Item<'a>>>,
    head: *mut Item<'a>,
    tail: *mut Item<'a>,
}

struct Item<'a> {
    name: Ident<'a>,
    value: Box<dyn Value<'a> + 'a>,
    next: *mut Item<'a>,
    prev: *mut Item<'a>,
}

impl fmt::Debug for Attributes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl Clone for Attributes<'_> {
    fn clone(&self) -> Self {
        let mut t = Self::with_capacity(self.indices.len());
        for (k, v) in self.iter() {
            t.insert(k.clone(), v.clone_boxed());
        }
        t
    }
}

impl<'a> Attributes<'a> {
    pub fn new() -> Self {
        Self {
            indices: HashMap::new(),
            head: 0 as _,
            tail: 0 as _,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            indices: HashMap::with_capacity(capacity),
            head: 0 as _,
            tail: 0 as _,
        }
    }

    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn get(&self, name: &str) -> Option<&Box<dyn Value<'a> + 'a>> {
        self.indices.get(name).map(|t| &t.value)
    }

    pub fn get_name_value(&self, name: &str) -> Option<(&Ident<'a>, &Box<dyn Value<'a> + 'a>)> {
        self.indices.get(name).map(|t| (&t.name, &t.value))
    }

    pub fn insert(&mut self, name: Ident<'a>, value: Box<dyn Value<'a> + 'a>) -> bool {
        let key = unsafe { &*(name.value.as_ref() as *const str) };
        match self.indices.entry(key) {
            std::collections::hash_map::Entry::Occupied(_) => false,
            std::collections::hash_map::Entry::Vacant(e) => {
                let mut item = Box::new(Item {
                    name,
                    value,
                    next: 0 as _,
                    prev: self.tail,
                });
                self.tail = item.as_mut() as _;
                unsafe {
                    if let Some(prev) = item.prev.as_mut() {
                        prev.next = self.tail;
                    } else {
                        self.head = self.tail;
                    }
                }
                e.insert(item);
                true
            }
        }
    }

    pub fn entry(&mut self, name: Ident<'a>) -> Entry<'_, 'a> {
        let key = unsafe { &*(name.value.as_ref() as *const str) };
        match self.indices.entry(key) {
            std::collections::hash_map::Entry::Occupied(e) => {
                Entry::Occupied(OccupiedEntry((&mut self.head, &mut self.tail), ManuallyDrop::new(e)))
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                Entry::Vacant(VacantEntry((&mut self.head, &mut self.tail), ManuallyDrop::new((name, e))))
            }
        }
    }

    pub fn iter(&self) -> Iter<'_, 'a> {
        Iter {
            item: self.head,
            len: self.indices.len(),
            phantom: PhantomData,
        }
    }
}

pub struct Iter<'r, 'a> {
    item: *mut Item<'a>,
    len: usize,
    phantom: PhantomData<&'r Attributes<'a>>,
}

impl<'r, 'a> Iterator for Iter<'r, 'a> {
    type Item = (&'r Ident<'a>, &'r Box<dyn Value<'a> + 'a>);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let Some(item) = self.item.as_mut() else { return None };
            self.item = item.next;
            self.len -= 1;
            Some((&item.name, &item.value))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'r, 'a> ExactSizeIterator for Iter<'r, 'a> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'r, 'a> FusedIterator for Iter<'r, 'a> {}

pub enum Entry<'r, 'a> {
    Occupied(OccupiedEntry<'r, 'a>),
    Vacant(VacantEntry<'r, 'a>),
}

pub struct OccupiedEntry<'r, 'a> ((&'r mut *mut Item<'a>, &'r mut *mut Item<'a>), ManuallyDrop<std::collections::hash_map::OccupiedEntry<'r, &'static str, Box<Item<'a>>>>);

impl<'r, 'a> OccupiedEntry<'r, 'a> {
    pub fn name(&self) -> &Ident<'a> {
        &self.1.get().name
    }

    pub fn remove(mut self) -> Box<dyn Value<'a> + 'a> {
        let entry = unsafe { ManuallyDrop::take(&mut self.1) };
        let item = entry.remove();
        unsafe {
            if let Some(prev) = item.prev.as_mut() {
                prev.next = item.next;
            } else {
                *self.0.0 = item.next;
            }
            if let Some(next) = item.next.as_mut() {
                next.prev = item.prev;
            } else {
                *self.0.1 = item.prev;
            }
        }
        forget(self);
        item.value
    }
}

impl<'r, 'a> Drop for OccupiedEntry<'r, 'a> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.1)
        }
    }
}

pub struct VacantEntry<'r, 'a> ((&'r mut *mut Item<'a>, &'r mut *mut Item<'a>), ManuallyDrop<(Ident<'a>, std::collections::hash_map::VacantEntry<'r, &'static str, Box<Item<'a>>>)>);

impl<'r, 'a> Drop for VacantEntry<'r, 'a> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.1)
        }
    }
}

impl<'r, 'a> VacantEntry<'r, 'a> {
    pub fn insert(mut self, value: Box<dyn Value<'a> + 'a>) -> &'r mut Box<dyn Value<'a> + 'a> {
        let (name, entry) = unsafe { ManuallyDrop::take(&mut self.1) };


        let mut item = Box::new(Item {
            name,
            value,
            next: 0 as _,
            prev: *self.0.1,
        });

        *self.0.1 = item.as_mut() as _;

        if let Some(prev) = unsafe { item.prev.as_mut() } {
            prev.next = *self.0.1;
        } else {
            *self.0.0 = *self.0.1;
        }

        forget(self);
        &mut entry.insert(item).value
    }
}