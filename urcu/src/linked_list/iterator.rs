use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::atomic::Ordering;

use crate::linked_list::raw::RcuListNode;
use crate::linked_list::{RcuListEntry, RcuListReader, RcuListWriter};
use crate::RcuContext;

/// An iterator over the nodes of an [`RcuList`].
///
/// [`RcuList`]: crate::linked_list::RcuList
pub struct RcuListIterator<T, O> {
    #[allow(dead_code)]
    reader: O,
    forward: bool,
    ptr: *const RcuListNode<T>,
}

impl<T, O> RcuListIterator<T, O> {
    pub fn new_forward(reader: O, ptr: *const RcuListNode<T>) -> Self {
        Self {
            reader,
            forward: true,
            ptr,
        }
    }

    pub fn new_reverse(reader: O, ptr: *const RcuListNode<T>) -> Self {
        Self {
            reader,
            forward: false,
            ptr,
        }
    }
}

impl<'a, T, C> Iterator for RcuListIterator<T, &'a RcuListReader<'a, T, C>>
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
                RcuListNode::next_node(self.ptr, Ordering::Acquire)
            } else {
                RcuListNode::prev_node(self.ptr, Ordering::Acquire)
            };

            Some(item)
        }
    }
}

impl<'a, T, C> Iterator for RcuListIterator<T, &'a RcuListWriter<T, C>> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.is_null() {
            return None;
        }

        // SAFETY: The pointer is non-null.
        unsafe {
            let item = &*self.ptr;

            self.ptr = if self.forward {
                RcuListNode::next_node(self.ptr, Ordering::Acquire)
            } else {
                RcuListNode::prev_node(self.ptr, Ordering::Acquire)
            };

            Some(item)
        }
    }
}

impl<'a, T, C> Iterator for RcuListIterator<T, &'a mut RcuListWriter<T, C>> {
    type Item = RcuListEntry<'a, T, C>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.is_null() {
            return None;
        }

        // SAFETY: The pointer is non-null.
        unsafe {
            let item = self.ptr as *mut RcuListNode<T>;

            self.ptr = if self.forward {
                RcuListNode::next_node(self.ptr, Ordering::Acquire)
            } else {
                RcuListNode::prev_node(self.ptr, Ordering::Acquire)
            };

            Some(RcuListEntry {
                node: NonNull::new_unchecked(item),
                list: self.reader.list.clone(),
                life: PhantomData,
            })
        }
    }
}
