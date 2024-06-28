use std::{ptr::NonNull, marker::PhantomData, iter::FusedIterator};

use super::{SharedListNode, List, ListNodePtr};



impl<T: ?Sized + SharedListNode<N>, const N: usize> List<T, N> {
    
    pub unsafe fn push_shared_front(&mut self, node: &T) {
        let node: ListNodePtr<T> = node.into();
        debug_assert!(node.shared_block_mut().enter_list());

        if self.head.is_null() {
            self.head = node;
            self.tail = node;
        } else {
            node.shared_block_mut().next = self.head;
            self.head.shared_block_mut().prev = node;
            self.head = node;
        }

        self.len += 1;
    }

    pub unsafe fn push_shared_back(&mut self, node: &T) {
        let node: ListNodePtr<T> = node.into();
        debug_assert!(node.shared_block_mut().enter_list());

        if self.tail.is_null() {
            self.head = node;
            self.tail = node;
        } else {
            node.shared_block_mut().prev = self.tail;
            self.tail.shared_block_mut().next = node;
            self.tail = node;
        }

        self.len += 1;
    }

    pub unsafe fn pop_shared_front(&mut self) -> Option<NonNull<T>> {
        if self.head.is_null() {
            None
        } else {
            let node = self.head;
            if self.head == self.tail {
                self.head.clr();
                self.tail.clr();
                debug_assert!(self.len == 1);
                self.len = 0;
            } else {
                let next = node.shared_block_mut().next.take();
                self.head = next;
                next.shared_block_mut().prev.clr();
                debug_assert!(self.len > 0);
                self.len -= 1;
            }
            debug_assert!(node.shared_block_mut().leave_list());
            Some(NonNull::new_unchecked(node.as_ptr()))
        }
    }

    pub unsafe fn pop_shared_back(&mut self) -> Option<NonNull<T>> {
        if self.tail.is_null() {
            None
        } else {
            let node = self.tail;
            if self.head == self.tail {
                self.head.clr();
                self.tail.clr();
                debug_assert!(self.len == 1);
                self.len = 0;
            } else {
                let prev = node.shared_block_mut().prev.take();
                self.tail = prev;
                prev.shared_block_mut().next.clr();
                debug_assert!(self.len > 0);
                self.len -= 1;
            }
            debug_assert!(node.shared_block_mut().leave_list());
            Some(NonNull::new_unchecked(node.as_ptr()))
        }
    }

    pub unsafe fn remove_shared(&mut self, node: &T) {
        debug_assert!(node.block_mut().leave_list());

        let node: ListNodePtr<T> = node.into();
        let mut block_mut = node.shared_block_mut();
        let prev = block_mut.prev.take();
        let next = block_mut.next.take();
        drop(block_mut);
        
        if prev.is_null() {
            debug_assert!(node == self.head);
            self.head = next;
            if next.is_null() {
                debug_assert!(node == self.tail);
                self.tail.clr();
            } else {
                next.shared_block_mut().prev.clr();
            }
        } else {
            prev.shared_block_mut().next = next;
            if next.is_null() {
                debug_assert!(node == self.tail);
                self.tail = prev;
            } else {
                next.shared_block_mut().prev = prev;
            }
        }
        debug_assert!(self.len > 0);
        self.len -= 1;
    }

    pub unsafe fn shared_iter(&self) -> SharedIter<T, N> {
        SharedIter { front: self.head, back: self.tail, phantom: PhantomData }
    }

    pub unsafe fn drain_shared(&mut self) -> DrainShared<T, N> {
        self.len = 0;
        DrainShared { front: self.head.take(), back: self.tail.take() }
    }

    pub unsafe fn drain_shared_filter<F: FnMut(&T) -> bool>(&mut self, filter: F) -> DrainSharedFilter<T, F, N> {
        let front = self.head;
        let back = self.tail;
        DrainSharedFilter { list: self, front, back, filter }
    }
}

pub struct SharedIter<'a, T: ?Sized, const N: usize> {
    front: ListNodePtr<T>,
    back: ListNodePtr<T>,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: ?Sized + SharedListNode<N>, const N: usize> Iterator for SharedIter<'a, T, N> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.front.is_null() {
            None
        } else {
            let node = self.front;
            if node == self.back {
                self.front.clr();
                self.back.clr();
            } else {
                self.front = unsafe { node.shared_block().next };
            }
            unsafe { Some(&*node.as_ptr()) }
        }
    }
}

impl<'a, T: ?Sized + SharedListNode<N>, const N: usize> DoubleEndedIterator for SharedIter<'a, T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back.is_null() {
            None 
        } else {
            let node = self.back;
            if node == self.front {
                self.front.clr();
                self.back.clr();
            } else {
                self.back = unsafe { node.shared_block().prev };
            }
            Some(unsafe { &*node.as_ptr() })
        }
    }
}

impl<'a, T: ?Sized + SharedListNode<N>, const N: usize> FusedIterator for SharedIter<'a, T, N> {
    
}

pub struct DrainShared<T: ?Sized + SharedListNode<N>, const N: usize> {
    front: ListNodePtr<T>,
    back: ListNodePtr<T>,
}

impl<T: ?Sized + SharedListNode<N>, const N: usize> Iterator for DrainShared<T, N> {
    type Item = NonNull<T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.front.is_null() {
            None
        } else {
            let node = self.front;
            if node == self.back {
                self.front.clr();
                self.back.clr();
            } else {
                unsafe {
                    self.front = node.shared_block_mut().next.take();
                    self.front.shared_block_mut().prev.clr();
                }
            }
            unsafe {
                debug_assert!(node.shared_block_mut().leave_list());
                Some(NonNull::new_unchecked(node.as_ptr()))
            }

        }
    }
}

impl<T: ?Sized + SharedListNode<N>, const N: usize> DoubleEndedIterator for DrainShared<T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back.is_null() {
            None 
        } else {
            let node = self.back;
            if node == self.front {
                self.front.clr();
                self.back.clr();
            } else {
                unsafe {
                    self.back = node.shared_block_mut().prev.take();
                    self.back.shared_block_mut().next.clr();
                }
            }
            unsafe {
                debug_assert!(node.shared_block_mut().leave_list());
                Some(NonNull::new_unchecked(node.as_ptr()))
            }
        }
    }
}

impl<T: ?Sized + SharedListNode<N>, const N: usize> FusedIterator for DrainShared<T, N> {
    
}

impl<T: ?Sized + SharedListNode<N>, const N: usize> Drop for DrainShared<T, N> {
    fn drop(&mut self) {
        while self.next().is_some() {

        }
    }
}


pub struct DrainSharedFilter<'a, T: ?Sized + SharedListNode<N>, F: FnMut(&T) -> bool, const N: usize> {
    list: &'a mut List<T, N>,
    front: ListNodePtr<T>,
    back: ListNodePtr<T>,
    filter: F,
}

impl<'a, T: ?Sized + SharedListNode<N>, F: FnMut(&T) -> bool, const N: usize> Iterator for DrainSharedFilter<'a, T, F, N> {
    type Item = NonNull<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.front.is_null() {
                break None;
            } else {
                let node = self.front;
                if node == self.back {
                    self.front.clr();
                    self.back.clr();
                } else {
                    unsafe {
                        self.front = node.shared_block().next;
                    }
                }

                if (self.filter)(unsafe { &*node.as_ptr() }) {
                    unsafe {
                        self.list.remove_shared(&*node.as_ptr());
                        break Some(NonNull::new_unchecked(node.as_ptr()));
                    }
                }
            }
        }
    }
}

impl<'a, T: ?Sized + SharedListNode<N>, F: FnMut(&T) -> bool, const N: usize> DoubleEndedIterator for DrainSharedFilter<'a, T, F, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if self.back.is_null() {
                break None;
            } else {
                let node = self.back;
                if node == self.front {
                    self.front.clr();
                    self.back.clr();
                } else {
                    unsafe {
                        self.back = node.shared_block().prev;
                    }
                }

                if (self.filter)(unsafe { &*node.as_ptr() }) {
                    unsafe {
                        self.list.remove_shared(&*node.as_ptr());
                        break Some(NonNull::new_unchecked(node.as_ptr()));
                    }
                }
            }
        }
    }
}

impl<'a, T: ?Sized + SharedListNode<N>, F: FnMut(&T) -> bool, const N: usize> FusedIterator for DrainSharedFilter<'a, T, F, N> {}

impl<'a, T: ?Sized + SharedListNode<N>, F: FnMut(&T) -> bool, const N: usize> Drop for DrainSharedFilter<'a, T, F, N> {
    fn drop(&mut self) {
        while self.next().is_some() {

        }
    }
}