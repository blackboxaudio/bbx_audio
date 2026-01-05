//! Stack-allocated vector with compile-time capacity.
//!
//! Provides a fixed-capacity vector that lives entirely on the stack,
//! suitable for use in realtime audio processing where heap allocations
//! must be avoided.

use core::{
    mem::MaybeUninit,
    ops::{Index, IndexMut},
    ptr,
};

/// A stack-allocated vector with compile-time capacity `N`.
///
/// This type provides `Vec`-like functionality without heap allocation,
/// making it suitable for use in audio processing hot paths where
/// allocation latency is unacceptable.
///
/// # Examples
///
/// ```
/// use bbx_core::StackVec;
///
/// let mut vec: StackVec<i32, 4> = StackVec::new();
/// vec.push(1).unwrap();
/// vec.push(2).unwrap();
///
/// assert_eq!(vec.len(), 2);
/// assert_eq!(vec[0], 1);
/// ```
pub struct StackVec<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> StackVec<T, N> {
    /// Creates a new empty `StackVec`.
    ///
    /// This is a const fn and can be used in const contexts.
    #[inline]
    pub const fn new() -> Self {
        Self {
            // SAFETY: An array of MaybeUninit<T> doesn't require initialization
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    /// Pushes a value onto the end of the vector.
    ///
    /// Returns `Ok(())` if successful, or `Err(value)` if the vector is full.
    #[inline]
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len >= N {
            return Err(value);
        }
        // SAFETY: We've verified len < N, so this index is valid
        unsafe {
            self.data.get_unchecked_mut(self.len).write(value);
        }
        self.len += 1;
        Ok(())
    }

    /// Pushes a value onto the end of the vector without bounds checking.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if the vector is full.
    /// In release mode, this will not panic but the behavior is safe
    /// due to the debug assertion.
    #[inline]
    pub fn push_unchecked(&mut self, value: T) {
        debug_assert!(self.len < N, "StackVec capacity exceeded");
        // SAFETY: Debug assertion ensures len < N in debug builds.
        // In release, we still check to prevent UB.
        if self.len < N {
            unsafe {
                self.data.get_unchecked_mut(self.len).write(value);
            }
            self.len += 1;
        }
    }

    /// Removes and returns the last element, or `None` if empty.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        // SAFETY: The element at len was initialized
        unsafe { Some(self.data.get_unchecked(self.len).assume_init_read()) }
    }

    /// Returns the number of elements in the vector.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the vector contains no elements.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns `true` if the vector is at capacity.
    #[inline]
    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    /// Returns the maximum capacity of the vector.
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Removes all elements from the vector.
    #[inline]
    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    /// Returns a slice of the initialized elements.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: Elements 0..len are initialized
        unsafe { core::slice::from_raw_parts(self.data.as_ptr() as *const T, self.len) }
    }

    /// Returns a mutable slice of the initialized elements.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: Elements 0..len are initialized
        unsafe { core::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, self.len) }
    }

    /// Returns a reference to the element at the given index, or `None` if out of bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            // SAFETY: We've verified index < len, so element is initialized
            unsafe { Some(&*self.data.get_unchecked(index).as_ptr()) }
        } else {
            None
        }
    }

    /// Returns a mutable reference to the element at the given index, or `None` if out of bounds.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            // SAFETY: We've verified index < len, so element is initialized
            unsafe { Some(&mut *self.data.get_unchecked_mut(index).as_mut_ptr()) }
        } else {
            None
        }
    }
}

impl<T, const N: usize> Default for StackVec<T, N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Drop for StackVec<T, N> {
    fn drop(&mut self) {
        for i in 0..self.len {
            // SAFETY: Elements 0..len are initialized
            unsafe {
                ptr::drop_in_place(self.data[i].as_mut_ptr());
            }
        }
    }
}

impl<T, const N: usize> Index<usize> for StackVec<T, N> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len, "index out of bounds");
        // SAFETY: We've verified index < len
        unsafe { &*self.data.get_unchecked(index).as_ptr() }
    }
}

impl<T, const N: usize> IndexMut<usize> for StackVec<T, N> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len, "index out of bounds");
        // SAFETY: We've verified index < len
        unsafe { &mut *self.data.get_unchecked_mut(index).as_mut_ptr() }
    }
}

impl<T: Clone, const N: usize> Clone for StackVec<T, N> {
    fn clone(&self) -> Self {
        let mut new_vec = Self::new();
        for item in self.as_slice() {
            // This won't fail since we're cloning from a vec with same capacity
            let _ = new_vec.push(item.clone());
        }
        new_vec
    }
}

// Note: Copy cannot be implemented for StackVec because it has a custom Drop.
// For Copy types, use StackVec::clone() which is efficient due to T: Copy.

// Iterator for owned values
pub struct StackVecIntoIter<T, const N: usize> {
    vec: StackVec<T, N>,
    index: usize,
}

impl<T, const N: usize> Iterator for StackVecIntoIter<T, N> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.len {
            // SAFETY: index < len means element is initialized
            let value = unsafe { self.vec.data.get_unchecked(self.index).assume_init_read() };
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.vec.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<T, const N: usize> ExactSizeIterator for StackVecIntoIter<T, N> {}

impl<T, const N: usize> Drop for StackVecIntoIter<T, N> {
    fn drop(&mut self) {
        for i in self.index..self.vec.len {
            // SAFETY: Elements index..len are still initialized
            unsafe {
                ptr::drop_in_place(self.vec.data[i].as_mut_ptr());
            }
        }
        // Prevent StackVec's Drop from running on already-handled elements
        self.vec.len = 0;
    }
}

impl<T, const N: usize> IntoIterator for StackVec<T, N> {
    type Item = T;
    type IntoIter = StackVecIntoIter<T, N>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        StackVecIntoIter { vec: self, index: 0 }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a StackVec<T, N> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut StackVec<T, N> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.as_mut_slice().iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;

    #[test]
    fn test_new_and_empty() {
        let vec: StackVec<i32, 4> = StackVec::new();
        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 4);
        assert!(!vec.is_full());
    }

    #[test]
    fn test_push_and_pop() {
        let mut vec: StackVec<i32, 4> = StackVec::new();

        assert!(vec.push(1).is_ok());
        assert!(vec.push(2).is_ok());
        assert!(vec.push(3).is_ok());
        assert_eq!(vec.len(), 3);

        assert_eq!(vec.pop(), Some(3));
        assert_eq!(vec.pop(), Some(2));
        assert_eq!(vec.pop(), Some(1));
        assert_eq!(vec.pop(), None);
        assert!(vec.is_empty());
    }

    #[test]
    fn test_capacity_limit() {
        let mut vec: StackVec<i32, 2> = StackVec::new();

        assert!(vec.push(1).is_ok());
        assert!(vec.push(2).is_ok());
        assert!(vec.is_full());

        // Should fail and return the value
        let result = vec.push(3);
        assert_eq!(result, Err(3));
        assert_eq!(vec.len(), 2);
    }

    #[test]
    fn test_push_unchecked() {
        let mut vec: StackVec<i32, 4> = StackVec::new();
        vec.push_unchecked(1);
        vec.push_unchecked(2);
        assert_eq!(vec.len(), 2);
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "StackVec capacity exceeded")]
    fn test_push_unchecked_overflow_panics_in_debug() {
        let mut vec: StackVec<i32, 1> = StackVec::new();
        vec.push_unchecked(1);
        vec.push_unchecked(2); // Should panic in debug mode
    }

    #[test]
    fn test_indexing() {
        let mut vec: StackVec<i32, 4> = StackVec::new();
        vec.push(10).unwrap();
        vec.push(20).unwrap();

        assert_eq!(vec[0], 10);
        assert_eq!(vec[1], 20);

        vec[0] = 100;
        assert_eq!(vec[0], 100);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_index_out_of_bounds() {
        let vec: StackVec<i32, 4> = StackVec::new();
        let _ = vec[0];
    }

    #[test]
    fn test_get() {
        let mut vec: StackVec<i32, 4> = StackVec::new();
        vec.push(1).unwrap();

        assert_eq!(vec.get(0), Some(&1));
        assert_eq!(vec.get(1), None);
    }

    #[test]
    fn test_get_mut() {
        let mut vec: StackVec<i32, 4> = StackVec::new();
        vec.push(1).unwrap();

        if let Some(val) = vec.get_mut(0) {
            *val = 42;
        }
        assert_eq!(vec[0], 42);
    }

    #[test]
    fn test_clear() {
        let mut vec: StackVec<i32, 4> = StackVec::new();
        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();

        vec.clear();
        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);

        // Can reuse after clear
        vec.push(42).unwrap();
        assert_eq!(vec[0], 42);
    }

    #[test]
    fn test_as_slice() {
        let mut vec: StackVec<i32, 4> = StackVec::new();
        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();

        assert_eq!(vec.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_as_mut_slice() {
        let mut vec: StackVec<i32, 4> = StackVec::new();
        vec.push(1).unwrap();
        vec.push(2).unwrap();

        vec.as_mut_slice()[0] = 10;
        assert_eq!(vec[0], 10);
    }

    #[test]
    fn test_iter_ref() {
        let mut vec: StackVec<i32, 4> = StackVec::new();
        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();

        let collected: Vec<_> = (&vec).into_iter().copied().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn test_iter_mut() {
        let mut vec: StackVec<i32, 4> = StackVec::new();
        vec.push(1).unwrap();
        vec.push(2).unwrap();

        for val in &mut vec {
            *val *= 2;
        }

        assert_eq!(vec[0], 2);
        assert_eq!(vec[1], 4);
    }

    #[test]
    fn test_into_iter() {
        let mut vec: StackVec<i32, 4> = StackVec::new();
        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();

        let collected: Vec<_> = vec.into_iter().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn test_clone() {
        let mut vec: StackVec<i32, 4> = StackVec::new();
        vec.push(1).unwrap();
        vec.push(2).unwrap();

        let cloned = vec.clone();
        assert_eq!(cloned.len(), 2);
        assert_eq!(cloned[0], 1);
        assert_eq!(cloned[1], 2);
    }

    #[test]
    fn test_drop_semantics_with_non_copy() {
        // Use Rc to track drop calls
        let counter = Rc::new(());

        {
            let mut vec: StackVec<Rc<()>, 4> = StackVec::new();
            vec.push(Rc::clone(&counter)).unwrap();
            vec.push(Rc::clone(&counter)).unwrap();
            vec.push(Rc::clone(&counter)).unwrap();

            // counter + 3 clones = 4 refs
            assert_eq!(Rc::strong_count(&counter), 4);
        }

        // After vec drops, only original counter remains
        assert_eq!(Rc::strong_count(&counter), 1);
    }

    #[test]
    fn test_drop_on_clear_with_non_copy() {
        let counter = Rc::new(());

        let mut vec: StackVec<Rc<()>, 4> = StackVec::new();
        vec.push(Rc::clone(&counter)).unwrap();
        vec.push(Rc::clone(&counter)).unwrap();

        assert_eq!(Rc::strong_count(&counter), 3);

        vec.clear();
        assert_eq!(Rc::strong_count(&counter), 1);
    }

    #[test]
    fn test_into_iter_drop_remaining() {
        let counter = Rc::new(());

        let mut vec: StackVec<Rc<()>, 4> = StackVec::new();
        vec.push(Rc::clone(&counter)).unwrap();
        vec.push(Rc::clone(&counter)).unwrap();
        vec.push(Rc::clone(&counter)).unwrap();

        assert_eq!(Rc::strong_count(&counter), 4);

        // Only consume first element
        let mut iter = vec.into_iter();
        drop(iter.next()); // Consume and immediately drop first element

        // Drop iter without consuming all elements
        drop(iter);

        // All elements should be dropped
        assert_eq!(Rc::strong_count(&counter), 1);
    }

    #[test]
    fn test_default() {
        let vec: StackVec<i32, 4> = StackVec::default();
        assert!(vec.is_empty());
    }

    #[test]
    fn test_size_hint() {
        let mut vec: StackVec<i32, 4> = StackVec::new();
        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();

        let iter = vec.into_iter();
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.len(), 3);
    }

    #[test]
    fn test_zero_capacity() {
        let vec: StackVec<i32, 0> = StackVec::new();
        assert!(vec.is_empty());
        assert!(vec.is_full());
        assert_eq!(vec.capacity(), 0);
    }
}
