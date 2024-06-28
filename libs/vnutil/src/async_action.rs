use std::{mem::{ManuallyDrop, transmute, forget}, task::{Context, Poll}, future::Future, pin::Pin, ptr::NonNull};



union RawInner<T1, T2> {
    async_fn: ManuallyDrop<T1>,
    future: ManuallyDrop<T2>,
}

trait Inner<'a, T: ?Sized, R>: Send {
    unsafe fn call(&mut self, obj: &'a mut T);
    unsafe fn poll(&mut self, cx: &mut Context<'_>) -> Poll<R>;
    unsafe fn drop_async_fn(&mut self);
    unsafe fn drop_future(&mut self);
}

impl<'a, T, R, F, FR> Inner<'a, T, R> for RawInner<F, FR>
where
    T: 'a + ?Sized,
    FR: 'a + Future<Output = R> + Send,
    F: FnOnce(&'a mut T) -> FR + Send,
{
    unsafe fn call(&mut self, obj: &'a mut T) {
        let f = ManuallyDrop::take(&mut self.async_fn);
        self.future = ManuallyDrop::new(f(obj));
    }
    unsafe fn poll(&mut self, cx: &mut Context<'_>) -> Poll<R> {
        Pin::new_unchecked(&mut *self.future).poll(cx)
    }
    unsafe fn drop_async_fn(&mut self) {
        ManuallyDrop::drop(&mut self.async_fn);
    }
    unsafe fn drop_future(&mut self) {
        ManuallyDrop::drop(&mut self.future);
    }
}

pub struct AsyncAction<T: ?Sized, R> {
    inner: NonNull<dyn for<'a> Inner<'a, T, R>>,
}

impl<T: ?Sized, R> AsyncAction<T, R> {
    pub fn new<'t, 'r, F, FR>(async_fn: F) -> Self
    where
        T: 't,
        FR: 't + Future<Output = R> + Send,
        F: 'r + FnOnce(&'t mut T) -> FR + Send,
        Self: 'r
    {
        let inner = Box::new(RawInner {
            async_fn: ManuallyDrop::new(async_fn),
        }) as Box<dyn Inner<T, R>>;
        Self {
            inner: unsafe { NonNull::new_unchecked(transmute(Box::into_raw(inner))) }
        }
    }

    pub fn into_future<'a>(self, obj: &'a mut T) -> AsyncActionFuture<'a, T, R> {
        unsafe {
            let mut inner = self.inner;
            forget(self);
            inner.as_mut().call(obj);

            AsyncActionFuture { inner }
        }
    }
}

unsafe impl<T: ?Sized, R> Send for AsyncAction<T, R> {}

impl<T: ?Sized, R> Drop for AsyncAction<T, R> {
    fn drop(&mut self) {
        unsafe {
            self.inner.as_mut().drop_async_fn();
            let _ = Box::from_raw(self.inner.as_ptr());
        }
    }
}

pub struct AsyncActionFuture<'a, T: ?Sized, R> {
    inner: NonNull<dyn Inner<'a, T, R>>,
}

unsafe impl<'a, T: ?Sized, R> Send for AsyncActionFuture<'a, T, R> {}

impl<'a, T: ?Sized, R> Drop for AsyncActionFuture<'a, T, R> {
    fn drop(&mut self) {
        unsafe {
            self.inner.as_mut().drop_future();
            let _ = Box::from_raw(self.inner.as_ptr());
        }
    }
}

impl<'a, T: ?Sized, R> Future for AsyncActionFuture<'a, T, R> {
    type Output = R;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { self.get_mut().inner.as_mut().poll(cx) }
    }
}