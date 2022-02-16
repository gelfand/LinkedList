use std::{
    borrow::Borrow,
    marker::PhantomData,
    sync::{
        atomic::{AtomicPtr, AtomicUsize},
        Arc,
    },
};

#[allow(dead_code)]
pub struct Node<T> {
    next: Arc<AtomicPtr<Option<Node<T>>>>,
    prev: Arc<AtomicPtr<Option<Node<T>>>>,
    value: T,
}

impl<T> Node<T> {
    pub fn new(value: T) -> Self {
        Self {
            next: Arc::new(AtomicPtr::new(std::ptr::null_mut())),
            prev: Arc::new(AtomicPtr::new(std::ptr::null_mut())),
            value,
        }
    }
}

impl<T> AsRef<Node<T>> for Node<T> {
    fn as_ref(&self) -> &Self {
        self
    }
}

pub struct LinkedList<T> {
    head: Arc<AtomicPtr<Option<Node<T>>>>,
    tail: Arc<AtomicPtr<Option<Node<T>>>>,
    len: Arc<AtomicUsize>,
    _marker: PhantomData<Box<T>>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        LinkedList {
            head: Arc::new(AtomicPtr::new(std::ptr::null_mut())),
            tail: Arc::new(AtomicPtr::new(std::ptr::null_mut())),
            len: Arc::new(AtomicUsize::new(0)),
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push_front(&self, value: T) {
        let node = Box::into_raw(Box::new(Some(Node {
            next: self.head.clone(),
            prev: Arc::new(AtomicPtr::new(std::ptr::null_mut())),
            value,
        })));

        if self
            .head
            .load(std::sync::atomic::Ordering::Relaxed)
            .is_null()
        {
            self.tail.store(node, std::sync::atomic::Ordering::Relaxed);
        } else {
            unsafe {
                self.head
                    .load(std::sync::atomic::Ordering::Relaxed)
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .prev
                    .store(node, std::sync::atomic::Ordering::Relaxed);
            }
        }
        self.head.store(node, std::sync::atomic::Ordering::Relaxed);
        self.len.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn push_back(&self, value: T) {
        let node = Box::into_raw(Box::new(Some(Node {
            next: Arc::new(AtomicPtr::new(std::ptr::null_mut())),
            prev: self.tail.clone(),
            value,
        })));

        if self
            .tail
            .load(std::sync::atomic::Ordering::Relaxed)
            .is_null()
        {
            self.head.store(node, std::sync::atomic::Ordering::Relaxed);
        } else {
            unsafe {
                self.tail
                    .load(std::sync::atomic::Ordering::Relaxed)
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .next
                    .store(node, std::sync::atomic::Ordering::Relaxed);
            }
        }
        self.tail.store(node, std::sync::atomic::Ordering::Relaxed);
        self.len.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn find(&self, value: &T) -> Option<&T>
    where
        T: PartialEq,
    {
        let mut curr = self.head.load(std::sync::atomic::Ordering::Relaxed);
        while !curr.is_null() {
            if unsafe { (*curr).as_ref().unwrap().value == *value } {
                return unsafe { Some(&(*curr).as_ref().unwrap().value) };
            }
            curr = unsafe {
                (*curr)
                    .as_ref()
                    .unwrap()
                    .next
                    .load(std::sync::atomic::Ordering::Relaxed)
            };
        }
        None
    }

    pub fn remove(&self, value: &T) -> Option<T>
    where
        // FIXME: Fix me, but how?
        T: PartialEq + Clone + Copy,
    {
        let mut curr = self.head.load(std::sync::atomic::Ordering::Relaxed);
        while !curr.is_null() {
            if unsafe { (*curr).as_ref().unwrap().value == *value } {
                let node = unsafe { (*curr).as_ref().unwrap() };
                let next = node.next.load(std::sync::atomic::Ordering::Relaxed);
                let prev = node.prev.load(std::sync::atomic::Ordering::Relaxed);
                if !next.is_null() {
                    unsafe {
                        next.as_ref()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .prev
                            .store(prev, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                if !prev.is_null() {
                    unsafe {
                        prev.as_ref()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .next
                            .store(next, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                self.len.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                return Some(node.value);
            }
            curr = unsafe {
                (*curr)
                    .as_ref()
                    .unwrap()
                    .next
                    .load(std::sync::atomic::Ordering::Relaxed)
            };
        }
        None
    }
}

mod tests {

    #[test]
    fn test_push_front() {
        use super::*;
        let list = LinkedList::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);
        list.push_front(4);
        unsafe {
            let mut i = 1;
            let mut current = list.tail.load(std::sync::atomic::Ordering::Relaxed);
            while let Some(node) = current.as_ref() {
                println!("{:?}", node.as_ref().unwrap().value);
                assert_eq!(node.as_ref().unwrap().value, i);
                i += 1;
                current = node
                    .as_ref()
                    .unwrap()
                    .prev
                    .load(std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    #[test]
    fn test_push_back() {
        use super::*;
        let list = LinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        list.push_back(4);
        unsafe {
            let mut i = 1;
            let mut current = list.head.load(std::sync::atomic::Ordering::Relaxed);
            while let Some(node) = current.as_ref() {
                println!("{:?}", node.as_ref().unwrap().value);
                assert_eq!(node.as_ref().unwrap().value, i);
                i += 1;
                current = node
                    .as_ref()
                    .unwrap()
                    .next
                    .load(std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    #[test]
    fn test_find() {
        use super::*;
        let list = LinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        list.push_back(4);
        assert_eq!(list.find(&1), Some(&1));
    }

    #[test]
    fn test_remove() {
        use super::*;
        let list = LinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        list.push_back(4);
        assert_eq!(list.remove(&1), Some(1));
        assert_eq!(list.remove(&2), Some(2));
        assert_eq!(list.remove(&3), Some(3));
        assert_eq!(list.remove(&4), Some(4));
    }
}
