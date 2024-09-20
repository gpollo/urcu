use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::atomic::Ordering;

use crate::linked_list::raw::Node;
use crate::linked_list::{Entry, Reader, Writer};
use crate::RcuContext;

/// An iterator over the nodes of an [`RcuList`].
///
/// [`RcuList`]: crate::linked_list::RcuList
pub struct Iter<T, O> {
    #[allow(dead_code)]
    reader: O,
    forward: bool,
    ptr: *const Node<T>,
}

impl<T, O> Iter<T, O> {
    pub fn new_forward(reader: O, ptr: *const Node<T>) -> Self {
        Self {
            reader,
            forward: true,
            ptr,
        }
    }

    pub fn new_reverse(reader: O, ptr: *const Node<T>) -> Self {
        Self {
            reader,
            forward: false,
            ptr,
        }
    }
}

impl<'a, T, C> Iterator for Iter<T, &'a Reader<'a, T, C>>
where
    C: RcuContext + 'a,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.is_null() {
            return None;
        }

        // SAFETY: The pointer is non-null.
        unsafe {
            let item = &*self.ptr;

            self.ptr = if self.forward {
                Node::next_node(self.ptr, Ordering::Acquire)
            } else {
                Node::prev_node(self.ptr, Ordering::Acquire)
            };

            Some(item)
        }
    }
}

impl<'a, T, C> Iterator for Iter<T, &'a Writer<T, C>> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.is_null() {
            return None;
        }

        // SAFETY: The pointer is non-null.
        unsafe {
            let item = &*self.ptr;

            self.ptr = if self.forward {
                Node::next_node(self.ptr, Ordering::Acquire)
            } else {
                Node::prev_node(self.ptr, Ordering::Acquire)
            };

            Some(item)
        }
    }
}

impl<'a, T, C> Iterator for Iter<T, &'a mut Writer<T, C>> {
    type Item = Entry<'a, T, C>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.is_null() {
            return None;
        }

        // SAFETY: The pointer is non-null.
        unsafe {
            let item = self.ptr as *mut Node<T>;

            self.ptr = if self.forward {
                Node::next_node(self.ptr, Ordering::Acquire)
            } else {
                Node::prev_node(self.ptr, Ordering::Acquire)
            };

            Some(Entry {
                node: NonNull::new_unchecked(item),
                list: self.reader.list.clone(),
                life: PhantomData,
            })
        }
    }
}
