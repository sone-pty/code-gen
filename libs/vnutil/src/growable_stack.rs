use std::{mem::{MaybeUninit, size_of}, ptr::{NonNull, drop_in_place}, marker::PhantomPinned, slice::{from_raw_parts_mut, from_raw_parts}, alloc::{alloc, dealloc, Layout, realloc, handle_alloc_error}, pin::Pin};



pub struct GrowableStack<T, const LEN: usize> {
    array: [MaybeUninit<T>; LEN],
    stack: NonNull<T>,
    capacity: usize,
    count: usize,
    _marker: PhantomPinned,
}

impl<T, const LEN: usize> Drop for GrowableStack<T, LEN> {
    fn drop(&mut self) {
        unsafe {
            let ptr = self.stack.as_ptr();
            drop_in_place(from_raw_parts_mut(ptr, self.count));
            if ptr != MaybeUninit::slice_as_mut_ptr(&mut self.array) {
                dealloc(ptr.cast(), Layout::array::<T>(self.capacity).unwrap());
            }
        }
    }
}


impl<T, const LEN: usize> GrowableStack<T, LEN> {
    pub fn with<R, F: FnOnce(Pin<&'_ mut GrowableStack<T, LEN>>) -> R>(f: F) -> R {
        let mut stack: GrowableStack<T, LEN> = GrowableStack {
            array: MaybeUninit::uninit_array(),
            stack: NonNull::dangling(),
            capacity: LEN,
            count: 0,
            _marker: PhantomPinned,
        };

        unsafe {
            let mut stack = Pin::new_unchecked(&mut stack);
            let stack_mut = stack.as_mut().get_unchecked_mut();
            stack_mut.stack = NonNull::new_unchecked(MaybeUninit::slice_as_mut_ptr(&mut stack_mut.array));
            f(stack)
        }
    }

    pub fn push(self: &mut Pin<&mut Self>, value: T) {
        unsafe { self.as_mut().get_unchecked_mut().inner_push(value); }
    }

    pub fn pop(self: &mut Pin<&mut Self>) -> Option<T> {
        unsafe { self.as_mut().get_unchecked_mut().inner_pop() }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { from_raw_parts(self.stack.as_ptr(), self.count) }
    }

    pub fn as_slice_mut<'a>(self: &'a mut Pin<&mut Self>) -> &'a mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut::<'a>(self.stack.as_ptr(), self.count)
        }
    }

    unsafe fn inner_push(&mut self, value: T) {
        if self.count == self.capacity {
            let capacity = self.capacity * 2;
            let mut ptr = self.stack.as_ptr();
            if ptr == MaybeUninit::slice_as_mut_ptr(self.array.as_mut()) {
                ptr = alloc(Layout::array::<T>(capacity).unwrap()).cast();
            } else {
                ptr = realloc(ptr.cast(), Layout::array::<T>(self.capacity).unwrap(), capacity * size_of::<T>()).cast();
            }
            if ptr.is_null() {
                handle_alloc_error(Layout::array::<T>(capacity).unwrap());
            }
            self.stack = NonNull::new_unchecked(ptr);
            self.capacity = capacity;
        }

        self.stack.as_ptr().add(self.count).write(value);
        self.count += 1;
    }

    unsafe fn inner_pop(&mut self) -> Option<T> {
        if self.count > 0 {
            self.count -= 1;
            Some(self.stack.as_ptr().add(self.count).read())
        } else {
            None
        }
    }
}