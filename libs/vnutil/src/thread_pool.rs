use std::{collections::VecDeque, sync::{atomic::{AtomicUsize, Ordering}, Arc}, thread::{JoinHandle, Builder, self}, num::NonZeroUsize};

use crate::lock_api::{Condvar, Mutex};



pub struct ThreadPool {
    inner: Arc<Inner>,
    handles: Vec<JoinHandle<()>>,
}

struct Inner {
    condvar: Condvar,
    context: Mutex<Context>,
}

struct Context {
    queue: VecDeque<Box<dyn FnOnce() + Send>>,
    quit: bool,
}

impl ThreadPool {
    pub fn new(count: NonZeroUsize) -> Self {
        let inner = Arc::new(Inner {
            condvar: Condvar::new(),
            context: Mutex::new(Context {
                queue: VecDeque::new(),
                quit: false,
            }),
        });
        let count = count.get();
        let mut handles = Vec::with_capacity(count);
        for i in 0..count {
            let inner = inner.clone();
            let handle = Builder::new().name(format!("pool-thread-{i}")).spawn(move || thread(inner)).unwrap();
            handles.push(handle);
        }
        Self {
            inner,
            handles,
        }
    }

    pub fn post<F: 'static + FnOnce() + Send>(&self, f: F) {
        let mut ctx = self.inner.context.lock().unwrap();
        ctx.queue.push_front(Box::new(f));
        self.inner.condvar.notify_one();
    }

    pub fn new_group(&self) -> Group {
        Group::new(self)
    }

}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        {
            let mut ctx = self.inner.context.lock().unwrap();
            ctx.quit = true;
            self.inner.condvar.notify_all();
        }
        for handle in self.handles.drain(..) {
            let _ = handle.join();
        }
    }
}

fn thread(inner: Arc<Inner>) {
    let mut ctx = inner.context.lock().unwrap();
    while !ctx.quit {
        if let Some(job) = ctx.queue.pop_back() {
            drop(ctx);
            job();
            ctx = inner.context.lock().unwrap();
        } else {
            ctx = inner.condvar.wait(ctx).unwrap();
        }
    }
}


pub struct Group<'a> {
    pool: &'a ThreadPool,
    count: Arc<AtomicUsize>,
}

impl<'a> Group<'a> {
    pub fn new(pool: &'a ThreadPool) -> Self {
        Self {
            pool,
            count: Arc::new(AtomicUsize::new(1)),
        }
    }

    pub fn post<F: 'static + FnOnce() + Send>(&self, f: F) {
        let count = self.count.clone();
        count.fetch_add(1, Ordering::Relaxed);
        let thread = thread::current();
        self.pool.post(move || {
            f();
            if count.fetch_sub(1, Ordering::Relaxed) == 1 {
                thread.unpark();
            }
        })
    }

    pub fn wait_all_done(self) {
        if self.count.fetch_sub(1, Ordering::Relaxed) != 1 {
            thread::park();
        }
    }
}