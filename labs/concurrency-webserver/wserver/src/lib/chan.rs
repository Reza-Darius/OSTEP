/*
* a simple thread safe queue using conditional variables
*/

#![allow(dead_code)]
use std::{collections::VecDeque, sync::Arc};

use parking_lot::{Condvar, Mutex};

use crate::handler::Job;

// cheap handle to a thread safe channel
#[derive(Default)]
pub struct Channel {
    inner: Arc<ChanInner>,
}

impl Clone for Channel {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[derive(Default)]
pub struct ChanInner {
    q: Mutex<VecDeque<Job>>,
    prod_cv: Condvar,
    cons_cv: Condvar,
}

impl Channel {
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

    pub fn enqueue_shortest_job(&self, job: Job) {
        let mut guard = self.inner.q.lock();
        loop {
            if guard.capacity() > guard.len() {
                if let Some(front) = guard.front()
                    && job.file_size.unwrap() < front.file_size.unwrap()
                {
                    guard.push_front(job);
                } else {
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

    /// blocks the thread until room is available for a value
    pub fn push_back(&self, value: Job) {
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
    pub fn push_front(&self, value: Job) {
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
    pub fn pop_front(&self) -> Job {
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

    /// blocks the thread until a value becomes available
    pub fn pop_back(&self) -> Job {
        let mut guard = self.inner.q.lock();
        loop {
            if let Some(item) = guard.pop_back() {
                self.inner.prod_cv.notify_one();
                return item;
            } else {
                // wait on empty queue
                self.inner.cons_cv.wait(&mut guard);
            }
        }
    }
}
