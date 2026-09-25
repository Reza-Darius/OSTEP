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

        let mut q = Queue {
            data: Vec::with_capacity(cap),
            head: 0,
            tail: 0,
            len: 0,
        };
        unsafe {
            q.data.set_len(cap);
        };
        q
    }

    pub fn push_front(&mut self, val: T) {
        if self.is_full() {
            panic!("queue is full")
        }

        if self.is_empty() {
            self.push_back(val);
            return;
        }

        self.head = self.head.overflowing_sub(1).0;
        let i = self.hi();
        self.data[i].write(val);
        self.len += 1;
    }

    pub fn push_back(&mut self, val: T) {
        if self.is_full() {
            panic!("queue is full")
        }
        let i = self.ti();
        self.data[i].write(val);
        self.tail = self.tail.overflowing_add(1).0;
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let i = self.hi();
        let element = unsafe {
            // swap out element with uninit maybeuninit
            std::mem::replace(&mut self.data[i], MaybeUninit::uninit()).assume_init()
        };
        self.head = self.head.overflowing_add(1).0;
        self.len -= 1;
        Some(element)
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        self.tail = self.tail.overflowing_sub(1).0;
        let i = self.ti();
        let element = unsafe {
            // swap out element with uninit maybeuninit
            std::mem::replace(&mut self.data[i], MaybeUninit::uninit()).assume_init()
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

    // head index
    fn hi(&self) -> usize {
        self.idx(self.head)
    }

    // tail index
    fn ti(&self) -> usize {
        self.idx(self.tail)
    }

    fn idx(&self, i: usize) -> usize {
        i & (self.data.capacity() - 1)
    }
}

impl<T> Drop for Queue<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

// rounds n to the nearest power of two except for when it would overflow or if n == 0
fn round_pow2(n: usize) -> usize {
    let mut x = n;

    if x == 0 {
        return 0;
    }

    // if the left-most bit is set we can only round down
    if (n & (1 << 63)) != 0 {
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
    #[test]
    fn new_queue_is_empty() {
        let q = Queue::<i32>::new(8);

        assert!(q.is_empty());
        assert!(!q.is_full());
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn push_back_and_pop_front_fifo() {
        let mut q = Queue::new(4);

        q.push_back(1);
        q.push_back(2);
        q.push_back(3);

        assert_eq!(q.len(), 3);
        assert_eq!(q.pop_front(), Some(1));
        assert_eq!(q.pop_front(), Some(2));
        assert_eq!(q.pop_front(), Some(3));
        assert_eq!(q.pop_front(), None);
        assert!(q.is_empty());
    }

    #[test]
    fn push_front_and_pop_back_lifo() {
        let mut q = Queue::new(4);

        q.push_front(1);
        q.push_front(2);
        q.push_front(3);

        assert_eq!(q.len(), 3);
        assert_eq!(q.pop_back(), Some(1));
        assert_eq!(q.pop_back(), Some(2));
        assert_eq!(q.pop_back(), Some(3));
        assert_eq!(q.pop_back(), None);
        assert!(q.is_empty());
    }

    #[test]
    fn push_back_and_pop_back() {
        let mut q = Queue::new(4);

        q.push_back(1);
        q.push_back(2);
        q.push_back(3);

        assert_eq!(q.pop_back(), Some(3));
        assert_eq!(q.pop_back(), Some(2));
        assert_eq!(q.pop_back(), Some(1));
        assert_eq!(q.pop_back(), None);
    }

    #[test]
    fn push_front_and_pop_front() {
        let mut q = Queue::new(4);

        q.push_front(1);
        q.push_front(2);
        q.push_front(3);

        assert_eq!(q.pop_front(), Some(3));
        assert_eq!(q.pop_front(), Some(2));
        assert_eq!(q.pop_front(), Some(1));
        assert_eq!(q.pop_front(), None);
    }

    #[test]
    fn queue_reports_full() {
        let mut q = Queue::new(4);

        q.push_back(1);
        q.push_back(2);
        q.push_back(3);
        q.push_back(4);

        assert!(q.is_full());
        assert_eq!(q.len(), 4);
    }

    #[test]
    #[should_panic]
    fn pushing_full_queue_panics() {
        let mut q = Queue::new(4);

        for i in 0..4 {
            q.push_back(i);
        }

        q.push_back(5);
    }

    #[test]
    fn queue_can_be_reused_after_emptying() {
        let mut q = Queue::new(4);

        q.push_back(1);
        q.push_back(2);

        assert_eq!(q.pop_front(), Some(1));
        assert_eq!(q.pop_front(), Some(2));
        assert!(q.is_empty());

        q.push_back(3);
        q.push_back(4);

        assert_eq!(q.pop_front(), Some(3));
        assert_eq!(q.pop_front(), Some(4));
        assert!(q.is_empty());
    }

    #[test]
    fn mixed_front_and_back_operations() {
        let mut q = Queue::new(8);

        q.push_back(1);
        q.push_back(2);
        q.push_front(0);
        q.push_back(3);
        q.push_front(-1);

        assert_eq!(q.len(), 5);

        assert_eq!(q.pop_front(), Some(-1));
        assert_eq!(q.pop_back(), Some(3));
        assert_eq!(q.pop_front(), Some(0));
        assert_eq!(q.pop_back(), Some(2));
        assert_eq!(q.pop_front(), Some(1));

        assert!(q.is_empty());
    }

    #[test]
    fn capacity_is_rounded_to_power_of_two() {
        let q = Queue::<i32>::new(10);

        assert_eq!(q.data.capacity(), 16);
    }

    #[test]
    fn non_copy_values_are_dropped_correctly() {
        use std::rc::Rc;

        let a = Rc::new(());
        let b = Rc::new(());

        let mut q = Queue::new(4);

        q.push_back(Rc::clone(&a));
        q.push_back(Rc::clone(&b));

        assert_eq!(Rc::strong_count(&a), 2);
        assert_eq!(Rc::strong_count(&b), 2);

        drop(q);

        assert_eq!(Rc::strong_count(&a), 1);
        assert_eq!(Rc::strong_count(&b), 1);
    }
}
