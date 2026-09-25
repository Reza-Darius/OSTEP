#![allow(dead_code)]

use std::{
    mem::MaybeUninit,
    sync::{Condvar, Mutex},
};

pub struct WorkerPool<T> {
    queue: Mutex<T>,
    cv: Condvar,
}

struct Queue<T> {
    data: Vec<MaybeUninit<T>>,
    head: usize,
    tail: usize,
    len: usize,
}

impl<T> Queue<T> {
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0);
        let cap = round_pow2(cap);
        debug_assert!(cap.is_power_of_two(), "cap needs to be power of two");

        Queue {
            data: Vec::with_capacity(cap),
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    pub fn push_front(&mut self, val: T) {
        if self.is_full() {
            panic!("queue is full")
        }

        if self.is_empty() {
            self.push_front(val);
            return;
        }

        self.head -= 1;
        self.data[self.head].write(val);
        self.len += 1;
    }

    pub fn push_back(&mut self, val: T) {
        if self.is_full() {
            panic!("queue is full")
        }
        self.data[self.tail].write(val);
        self.tail += 1;
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let element = unsafe {
            // swap out element with uninit maybeuninit
            std::mem::replace(&mut self.data[self.head], MaybeUninit::uninit()).assume_init()
        };
        self.head += 1;
        self.len -= 1;
        Some(element)
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        self.tail -= 1;
        let element = unsafe {
            // swap out element with uninit maybeuninit
            std::mem::replace(&mut self.data[self.tail], MaybeUninit::uninit()).assume_init()
        };
        self.len -= 1;
        Some(element)
    }

    pub fn is_full(&self) -> bool {
        self.len == self.data.capacity()
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len
    }

    fn idx(&self, i: usize) -> usize {
        i & (self.data.capacity() - 1)
    }
}

// rounds n to the nearest power of two except for when it would overflow or if n == 0
fn round_pow2(n: usize) -> usize {
    let mut x = n;

    if x == 0 {
        return 0;
    }

    // if the left-most bit is set we can only round down
    if (n & (1 << 63)) > 0 {
        return 1 << 63;
    }

    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    x |= x >> 32;

    // x is now 2^n - 1
    let msb = (x + 1) >> 1;

    // round to nearest multiple of msb
    (n + (msb - 1)) & !(msb - 1)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rounding_pow() {
        assert_eq!(round_pow2(10), 16);
        assert_eq!(round_pow2(8), 8);
        assert_eq!(round_pow2(10), 16);
        assert_eq!(round_pow2(0), 0);
        assert_eq!(round_pow2((1 << 63) + 10), 1 << 63);
        assert_eq!(round_pow2(1000), 1024);
    }
}
