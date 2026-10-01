/*
* a simple thread safe queue using conditional variables
*/

#![allow(dead_code)]
use std::{collections::VecDeque, sync::Arc};

use parking_lot::{Condvar, Mutex};

use crate::handler::SffJob;

// cheap handle to a thread safe channel
#[derive(Default)]
pub struct Channel<T> {
    inner: Arc<ChanInner<T>>,
}

impl<T> Clone for Channel<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[derive(Default)]
pub struct ChanInner<T> {
    q: Mutex<VecDeque<T>>,
    prod_cv: Condvar,
    cons_cv: Condvar,
}

impl<T> Channel<T> {
    pub fn new(cap: usize) -> Self {
        Channel {
            inner: ChanInner {
                q: Mutex::new(VecDeque::with_capacity(cap)),
                prod_cv: Condvar::new(),
                cons_cv: Condvar::new(),
            }
            .into(),
        }
    }

    /// blocks the thread until room is available for a value
    pub fn push_back(&self, value: T) {
        let mut guard = self.inner.q.lock();
        loop {
            if guard.capacity() > guard.len() {
                guard.push_back(value);
                self.inner.cons_cv.notify_one();
                return;
            } else {
                // wait on full queue
                self.inner.prod_cv.wait(&mut guard);
            }
        }
    }

    /// blocks the thread until room is available for a value
    pub fn push_front(&self, value: T) {
        let mut guard = self.inner.q.lock();
        loop {
            if guard.capacity() > guard.len() {
                guard.push_front(value);
                self.inner.cons_cv.notify_one();
                return;
            } else {
                // wait on full queue
                self.inner.prod_cv.wait(&mut guard);
            }
        }
    }

    /// blocks the thread until a value becomes available
    pub fn pop_front(&self) -> T {
        let mut guard = self.inner.q.lock();
        loop {
            if let Some(item) = guard.pop_front() {
                self.inner.prod_cv.notify_one();
                return item;
            } else {
                // wait on empty queue
                self.inner.cons_cv.wait(&mut guard);
            }
        }
    }
}

impl Channel<SffJob> {
    pub fn enqueue_shortest_job(&self, job: SffJob) {
        let mut guard = self.inner.q.lock();
        loop {
            if guard.capacity() > guard.len() {
                if let Some(front) = guard.front()
                    && job.file_size < front.file_size
                {
                    eprintln!("pushing front");
                    guard.push_front(job);
                } else {
                    eprintln!("pushing back");
                    guard.push_back(job);
                }
                self.inner.cons_cv.notify_one();
                return;
            } else {
                // wait on full queue
                self.inner.prod_cv.wait(&mut guard);
            }
        }
    }
}
