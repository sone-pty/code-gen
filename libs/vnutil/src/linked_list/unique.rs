use std::{ptr::NonNull, marker::PhantomData, iter::FusedIterator};

use super::{UniqueListNode, List, ListNodePtr};

impl<T: ?Sized + UniqueListNode<N>, const N: usize> List<T, N> {
    pub unsafe fn push_unique_front(&mut self, node: &mut T) {
        let node: ListNodePtr<T> = node.into();
        debug_assert!(node.unique_block_mut().enter_list());

        if self.head.is_null() {
            self.head = node;
            self.tail = node;
        } else {
            node.unique_block_mut().next = self.head;
            self.head.unique_block_mut().prev = node;
            self.head = node;
        }

        self.len += 1;
    }

    pub unsafe fn push_unique_back(&mut self, node: &mut T) {
        let node: ListNodePtr<T> = node.into();
        debug_assert!(node.unique_block_mut().enter_list());

        if self.tail.is_null() {
            self.head = node;
            self.tail = node;
        } else {
            node.unique_block_mut().prev = self.tail;
            self.tail.unique_block_mut().next = node;
            self.tail = node;
        }

        self.len += 1;
    }

    pub unsafe fn pop_unique_front(&mut self) -> Option<NonNull<T>> {
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
                let next = node.unique_block_mut().next.take();
                self.head = next;
                next.unique_block_mut().prev.clr();
                debug_assert!(self.len > 0);
                self.len -= 1;
            }
            debug_assert!(node.unique_block_mut().leave_list());
            Some(NonNull::new_unchecked(node.as_ptr()))
        }
    }

    pub unsafe fn pop_unique_back(&mut self) -> Option<NonNull<T>> {
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
                let prev = node.unique_block_mut().prev.take();
                self.tail = prev;
                prev.unique_block_mut().next.clr();
                debug_assert!(self.len > 0);
                self.len -= 1;
            }
            debug_assert!(node.unique_block_mut().leave_list());
            Some(NonNull::new_unchecked(node.as_ptr()))
        }
    }

    pub unsafe fn remove_unique(&mut self, node: &mut T) {
        debug_assert!(node.block_mut().leave_list());

        let node: ListNodePtr<T> = node.into();
        let block_mut = node.unique_block_mut();
        let prev = block_mut.prev.take();
        let next = block_mut.next.take();
        if prev.is_null() {
            debug_assert!(node == self.head);
            self.head = next;
            if next.is_null() {
                debug_assert!(node == self.tail);
                self.tail.clr();
            } else {
                next.unique_block_mut().prev.clr();
            }
        } else {
            prev.unique_block_mut().next = next;
            if next.is_null() {
                debug_assert!(node == self.tail);
                self.tail = prev;
            } else {
                next.unique_block_mut().prev = prev;
            }
        }

        self.len -= 1;
    }

    pub unsafe fn unique_iter(&self) -> UniqueIter<T, N> {
        UniqueIter { front: self.head, back: self.tail, phantom: PhantomData }
    }

    pub unsafe fn unique_iter_mut(&mut self) -> UniqueIterMut<T, N> {
        UniqueIterMut { front: self.head, back: self.tail, phantom: PhantomData }
    }

    pub unsafe fn drain_unique(&mut self) -> DrainUnique<T, N> {
        self.len = 0;
        DrainUnique { front: self.head.take(), back: self.tail.take() }
    }

    pub unsafe fn drain_unique_filter<F: FnMut(&mut T) -> bool>(&mut self, filter: F) -> DrainUniqueFilter<T, F, N> {
        let front = self.head;
        let back = self.tail;
        DrainUniqueFilter { list: self, front, back, filter }
    }
}

pub struct UniqueIter<'a, T: ?Sized, const N: usize> {
    front: ListNodePtr<T>,
    back: ListNodePtr<T>,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: ?Sized + UniqueListNode<N>, const N: usize> Iterator for UniqueIter<'a, T, N> {
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
                self.front = unsafe { node.unique_block().next };
            }
            unsafe { Some(&*node.as_ptr()) }
        }
    }
}

impl<'a, T: ?Sized + UniqueListNode<N>, const N: usize> DoubleEndedIterator for UniqueIter<'a, T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back.is_null() {
            None 
        } else {
            let node = self.back;
            if node == self.front {
                self.front.clr();
                self.back.clr();
            } else {
                self.back = unsafe { node.unique_block().prev };
            }
            Some(unsafe { &*node.as_ptr() })
        }
    }
}

impl<'a, T: ?Sized + UniqueListNode<N>, const N: usize> FusedIterator for UniqueIter<'a, T, N> {
    
}

pub struct UniqueIterMut<'a, T: ?Sized, const N: usize> {
    front: ListNodePtr<T>,
    back: ListNodePtr<T>,
    phantom: PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized + UniqueListNode<N>, const N: usize> Iterator for UniqueIterMut<'a, T, N> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.front.is_null() {
            None
        } else {
            let node = self.front;
            if node == self.back {
                self.front.clr();
                self.back.clr();
            } else {
                self.front = unsafe { node.unique_block().next };
            }
            unsafe { Some(&mut *node.as_ptr()) }
        }
    }
}

impl<'a, T: ?Sized + UniqueListNode<N>, const N: usize> DoubleEndedIterator for UniqueIterMut<'a, T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back.is_null() {
            None 
        } else {
            let node = self.back;
            if node == self.front {
                self.front.clr();
                self.back.clr();
            } else {
                self.back = unsafe { node.unique_block().prev };
            }
            Some(unsafe { &mut *node.as_ptr() })
        }
    }
}

impl<'a, T: ?Sized + UniqueListNode<N>, const N: usize> FusedIterator for UniqueIterMut<'a, T, N> {
    
}

pub struct DrainUnique<T: ?Sized + UniqueListNode<N>, const N: usize> {
    front: ListNodePtr<T>,
    back: ListNodePtr<T>,
}

impl<T: ?Sized + UniqueListNode<N>, const N: usize> Iterator for DrainUnique<T, N> {
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
                    self.front = node.unique_block_mut().next.take();
                    self.front.unique_block_mut().prev.clr();
                }
            }
            unsafe {
                debug_assert!(node.unique_block_mut().leave_list());
                Some(NonNull::new_unchecked(node.as_ptr()))
            }

        }
    }
}

impl<T: ?Sized + UniqueListNode<N>, const N: usize> DoubleEndedIterator for DrainUnique<T, N> {
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
                    self.back = node.unique_block_mut().prev.take();
                    self.back.unique_block_mut().next.clr();
                }
            }
            unsafe {
                debug_assert!(node.unique_block_mut().leave_list());
                Some(NonNull::new_unchecked(node.as_ptr()))
            }
        }
    }
}

impl<T: ?Sized + UniqueListNode<N>, const N: usize> FusedIterator for DrainUnique<T, N> {
    
}

impl<T: ?Sized + UniqueListNode<N>, const N: usize> Drop for DrainUnique<T, N> {
    fn drop(&mut self) {
        while self.next().is_some() {

        }
    }
}

pub struct DrainUniqueFilter<'a, T: ?Sized + UniqueListNode<N>, F: FnMut(&mut T) -> bool, const N: usize> {
    list: &'a mut List<T, N>,
    front: ListNodePtr<T>,
    back: ListNodePtr<T>,
    filter: F,
}

impl<'a, T: ?Sized + UniqueListNode<N>, F: FnMut(&mut T) -> bool, const N: usize> Iterator for DrainUniqueFilter<'a, T, F, N> {
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
                        self.front = node.unique_block().next;
                    }
                }

                if (self.filter)(unsafe { &mut *node.as_ptr() }) {
                    unsafe {
                        self.list.remove_unique(&mut *node.as_ptr());
                        break Some(NonNull::new_unchecked(node.as_ptr()));
                    }
                }
            }
        }
    }
}

impl<'a, T: ?Sized + UniqueListNode<N>, F: FnMut(&mut T) -> bool, const N: usize> DoubleEndedIterator for DrainUniqueFilter<'a, T, F, N> {
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
                        self.back = node.unique_block().prev;
                    }
                }

                if (self.filter)(unsafe { &mut *node.as_ptr() }) {
                    unsafe {
                        self.list.remove_unique(&mut *node.as_ptr());
                        break Some(NonNull::new_unchecked(node.as_ptr()));
                    }
                }
            }
        }
    }
}

impl<'a, T: ?Sized + UniqueListNode<N>, F: FnMut(&mut T) -> bool, const N: usize> FusedIterator for DrainUniqueFilter<'a, T, F, N> {}

impl<'a, T: ?Sized + UniqueListNode<N>, F: FnMut(&mut T) -> bool, const N: usize> Drop for DrainUniqueFilter<'a, T, F, N> {
    fn drop(&mut self) {
        while self.next().is_some() {

        }
    }
}