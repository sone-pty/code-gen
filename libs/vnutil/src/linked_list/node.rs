use std::{ptr::{Pointee, from_raw_parts_mut, NonNull}, mem::{MaybeUninit, replace}, ops::{Deref, DerefMut}};


pub(super) struct ListNodePtr<T: ?Sized> (*mut (), <T as Pointee>::Metadata);

impl<T: ?Sized> Clone for ListNodePtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for ListNodePtr<T> {}

impl<T: ?Sized> ListNodePtr<T> {
    pub fn clr(&mut self) {
        *self = unsafe { MaybeUninit::zeroed().assume_init() };
    }

    pub fn is_null(self) -> bool {
        self.0 == 0_usize as _
    }

    pub fn as_ptr(self) -> *mut T {
        from_raw_parts_mut::<T>(self.0, self.1)
    }

    pub fn take(&mut self) -> Self {
        replace(self, unsafe { MaybeUninit::zeroed().assume_init() })
    }
}

impl<T: ?Sized> PartialEq for ListNodePtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: ?Sized> Eq for ListNodePtr<T> {}

impl<T: ?Sized> From<&mut T> for ListNodePtr<T> {
    fn from(value: &mut T) -> Self {
        let (p, m) = (value as *mut T).to_raw_parts();
        Self (p, m)
    }
}

impl<T: ?Sized> From<&T> for ListNodePtr<T> {
    fn from(value: &T) -> Self {
        let (p, m) = (value as *const T).to_raw_parts();
        Self (p as _, m)
    }
}

impl <T: ?Sized> ListNodePtr<T> {
    pub unsafe fn unique_block<'a, const N: usize>(self) -> &'a ListNodeBlock<T>
    where
        T: UniqueListNode<N>,
    {
        (*self.as_ptr()).block()
    }

    pub unsafe fn unique_block_mut<'a, const N: usize>(self) -> &'a mut ListNodeBlock<T>
    where
        T: UniqueListNode<N>,
    {
        (*self.as_ptr()).block_mut()
    }

    pub unsafe fn shared_block<'a, const N: usize>(self) -> T::BlockRef<'a>
    where
        T: SharedListNode<N>,
    {
        (*self.as_ptr()).block()
    }
    pub unsafe fn shared_block_mut<'a, const N: usize>(self) -> T::BlockMut<'a>
    where
        T: SharedListNode<N>,
    {
        (*self.as_ptr()).block_mut()
    }
}

pub struct ListNodeBlock<T: ?Sized> {
    pub(super) prev: ListNodePtr<T>,
    pub(super) next: ListNodePtr<T>,
    #[cfg(debug_assertions)]
    list: bool,
}

impl<T: ?Sized> Default for ListNodeBlock<T> {
    fn default() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() } 
    }
}


impl<T: ?Sized> ListNodeBlock<T> {
    #[cfg(debug_assertions)]
    pub(super) fn enter_list(&mut self) -> bool {
        if self.list {
            false
        } else {
            self.list = true;
            true
        }
    }

    #[cfg(not(debug_assertions))]
    pub(super) fn enter_list(&mut self) -> bool {
        unreachable!()
    }

    #[cfg(debug_assertions)]
    pub(super) fn leave_list(&mut self) -> bool {
        if self.list {
            self.list = false;
            true
        } else {
            false
        }
    }

    #[cfg(not(debug_assertions))]
    pub(super) fn leave_list(&mut self) -> bool {
        unreachable!()
    }

    pub fn prev(&self) -> Option<NonNull<T>> {
        if self.prev.is_null() {
            None
        } else {
            Some(unsafe { NonNull::new_unchecked(self.prev.as_ptr()) })
        }
    }

    pub fn next(&self) -> Option<NonNull<T>> {
        if self.next.is_null() {
            None
        } else {
            Some(unsafe { NonNull::new_unchecked(self.next.as_ptr()) })
        }
    }
}

pub trait UniqueListNode<const N: usize = 0> {
    fn block(&self) -> &ListNodeBlock<Self>;
    fn block_mut(&mut self) -> &mut ListNodeBlock<Self>;
}

pub trait SharedListNode<const N: usize = 0> {
    type BlockRef<'a>: Deref<Target = ListNodeBlock<Self>> where Self: 'a;
    type BlockMut<'a>: DerefMut<Target = ListNodeBlock<Self>> where Self: 'a;
    fn block(&self) -> Self::BlockRef<'_>;
    fn block_mut(&self) -> Self::BlockMut<'_>;
}