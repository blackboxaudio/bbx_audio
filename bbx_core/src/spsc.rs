//! Lock-free single-producer single-consumer ring buffer.
//!
//! Provides a realtime-safe channel for inter-thread communication,
//! suitable for audio thread to I/O thread communication where
//! blocking is unacceptable.

use core::{cell::UnsafeCell, mem::MaybeUninit};
#[cfg(not(loom))]
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[cfg(loom)]
use loom::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

/// Cache-line padded wrapper to prevent false sharing.
///
/// On most modern x86/ARM CPUs, cache lines are 64 bytes.
/// Padding head and tail to separate cache lines prevents
/// false sharing between producer and consumer threads.
#[repr(align(64))]
struct CachePadded<T>(T);

impl<T> CachePadded<T> {
    const fn new(value: T) -> Self {
        CachePadded(value)
    }
}

impl<T> core::ops::Deref for CachePadded<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Internal shared state for the ring buffer.
struct SpscRingBufferInner<T> {
    buffer: Box<[UnsafeCell<MaybeUninit<T>>]>,
    capacity: usize,
    mask: usize,
    head: CachePadded<AtomicUsize>, // Write position (producer)
    tail: CachePadded<AtomicUsize>, // Read position (consumer)
}

// SAFETY: SpscRingBufferInner can be shared between threads if T: Send
// because access is synchronized through atomic operations.
unsafe impl<T: Send> Send for SpscRingBufferInner<T> {}
unsafe impl<T: Send> Sync for SpscRingBufferInner<T> {}

impl<T> SpscRingBufferInner<T> {
    fn new(capacity: usize) -> Self {
        // Round up to next power of 2
        let capacity = capacity.next_power_of_two().max(1);
        let mask = capacity - 1;

        // Allocate buffer
        let buffer: Vec<UnsafeCell<MaybeUninit<T>>> =
            (0..capacity).map(|_| UnsafeCell::new(MaybeUninit::uninit())).collect();

        Self {
            buffer: buffer.into_boxed_slice(),
            capacity,
            mask,
            head: CachePadded::new(AtomicUsize::new(0)),
            tail: CachePadded::new(AtomicUsize::new(0)),
        }
    }
}

impl<T> Drop for SpscRingBufferInner<T> {
    fn drop(&mut self) {
        // Drop any remaining items in the buffer
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);

        for i in tail..head {
            let index = i & self.mask;
            // SAFETY: Elements between tail and head are initialized
            unsafe {
                let ptr = (*self.buffer[index].get()).as_mut_ptr();
                core::ptr::drop_in_place(ptr);
            }
        }
    }
}

/// Factory for creating producer/consumer pairs.
pub struct SpscRingBuffer;

impl SpscRingBuffer {
    /// Creates a new SPSC ring buffer with the given capacity.
    ///
    /// The actual capacity will be rounded up to the next power of 2.
    /// Returns a `(Producer, Consumer)` pair for inter-thread communication.
    ///
    /// # Examples
    ///
    /// ```
    /// use bbx_core::spsc::SpscRingBuffer;
    ///
    /// let (mut producer, mut consumer) = SpscRingBuffer::new::<i32>(4);
    ///
    /// producer.try_push(42).unwrap();
    /// assert_eq!(consumer.try_pop(), Some(42));
    /// ```
    #[allow(clippy::new_ret_no_self)]
    pub fn new<T>(capacity: usize) -> (Producer<T>, Consumer<T>) {
        let inner = Arc::new(SpscRingBufferInner::new(capacity));
        (
            Producer {
                inner: Arc::clone(&inner),
            },
            Consumer { inner },
        )
    }
}

/// Producer handle for pushing items into the ring buffer.
///
/// This type is `Send` but not `Clone` - only one producer should exist.
pub struct Producer<T> {
    inner: Arc<SpscRingBufferInner<T>>,
}

// SAFETY: Producer can be sent to another thread if T: Send
unsafe impl<T: Send> Send for Producer<T> {}

impl<T> Producer<T> {
    /// Attempts to push a value into the buffer.
    ///
    /// Returns `Ok(())` if successful, or `Err(value)` if the buffer is full.
    /// This operation is lock-free and will never block.
    #[inline]
    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        let head = self.inner.head.load(Ordering::Relaxed);
        let tail = self.inner.tail.load(Ordering::Acquire);

        if head.wrapping_sub(tail) >= self.inner.capacity {
            return Err(value);
        }

        let index = head & self.inner.mask;
        // SAFETY: We've verified there's space, and only producer writes to this slot
        unsafe {
            (*self.inner.buffer[index].get()).write(value);
        }

        self.inner.head.store(head.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    /// Returns the number of items currently in the buffer.
    ///
    /// This is an approximate count and may be stale by the time it's used.
    #[inline]
    pub fn len(&self) -> usize {
        let head = self.inner.head.load(Ordering::Relaxed);
        let tail = self.inner.tail.load(Ordering::Relaxed);
        head.wrapping_sub(tail)
    }

    /// Returns `true` if the buffer is full.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len() >= self.inner.capacity
    }

    /// Returns `true` if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the capacity of the buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity
    }
}

/// Consumer handle for popping items from the ring buffer.
///
/// This type is `Send` but not `Clone` - only one consumer should exist.
pub struct Consumer<T> {
    inner: Arc<SpscRingBufferInner<T>>,
}

// SAFETY: Consumer can be sent to another thread if T: Send
unsafe impl<T: Send> Send for Consumer<T> {}

impl<T> Consumer<T> {
    /// Attempts to pop a value from the buffer.
    ///
    /// Returns `Some(value)` if successful, or `None` if the buffer is empty.
    /// This operation is lock-free and will never block.
    #[inline]
    pub fn try_pop(&mut self) -> Option<T> {
        let tail = self.inner.tail.load(Ordering::Relaxed);
        let head = self.inner.head.load(Ordering::Acquire);

        if tail >= head {
            return None;
        }

        let index = tail & self.inner.mask;
        // SAFETY: We've verified there's data, and only consumer reads from this slot
        let value = unsafe { (*self.inner.buffer[index].get()).assume_init_read() };

        self.inner.tail.store(tail.wrapping_add(1), Ordering::Release);
        Some(value)
    }

    /// Returns the number of items currently in the buffer.
    ///
    /// This is an approximate count and may be stale by the time it's used.
    #[inline]
    pub fn len(&self) -> usize {
        let head = self.inner.head.load(Ordering::Relaxed);
        let tail = self.inner.tail.load(Ordering::Relaxed);
        head.wrapping_sub(tail)
    }

    /// Returns `true` if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if the buffer is full.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len() >= self.inner.capacity
    }

    /// Returns the capacity of the buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use std::{rc::Rc, thread};

    use super::*;

    #[test]
    fn test_basic_push_pop() {
        let (mut producer, mut consumer) = SpscRingBuffer::new::<i32>(4);

        assert!(producer.try_push(1).is_ok());
        assert!(producer.try_push(2).is_ok());
        assert!(producer.try_push(3).is_ok());

        assert_eq!(consumer.try_pop(), Some(1));
        assert_eq!(consumer.try_pop(), Some(2));
        assert_eq!(consumer.try_pop(), Some(3));
        assert_eq!(consumer.try_pop(), None);
    }

    #[test]
    fn test_empty_buffer() {
        let (_producer, mut consumer) = SpscRingBuffer::new::<i32>(4);

        assert!(consumer.is_empty());
        assert_eq!(consumer.try_pop(), None);
    }

    #[test]
    fn test_full_buffer() {
        let (mut producer, _consumer) = SpscRingBuffer::new::<i32>(4);

        // Capacity is rounded to power of 2, so 4 elements
        assert!(producer.try_push(1).is_ok());
        assert!(producer.try_push(2).is_ok());
        assert!(producer.try_push(3).is_ok());
        assert!(producer.try_push(4).is_ok());
        assert!(producer.is_full());

        // Should fail
        assert_eq!(producer.try_push(5), Err(5));
    }

    #[test]
    fn test_capacity_rounding() {
        // Request 3, should round to 4
        let (producer, _consumer) = SpscRingBuffer::new::<i32>(3);
        assert_eq!(producer.capacity(), 4);

        // Request 5, should round to 8
        let (producer, _consumer) = SpscRingBuffer::new::<i32>(5);
        assert_eq!(producer.capacity(), 8);

        // Request 0, should become 1
        let (producer, _consumer) = SpscRingBuffer::new::<i32>(0);
        assert_eq!(producer.capacity(), 1);
    }

    #[test]
    fn test_wraparound() {
        let (mut producer, mut consumer) = SpscRingBuffer::new::<i32>(4);

        // Fill and empty multiple times to test wraparound
        for round in 0..10 {
            for i in 0..4 {
                assert!(producer.try_push(round * 10 + i).is_ok());
            }

            for i in 0..4 {
                assert_eq!(consumer.try_pop(), Some(round * 10 + i));
            }
        }
    }

    #[test]
    fn test_len() {
        let (mut producer, mut consumer) = SpscRingBuffer::new::<i32>(4);

        assert_eq!(producer.len(), 0);
        assert_eq!(consumer.len(), 0);

        producer.try_push(1).unwrap();
        assert_eq!(producer.len(), 1);
        assert_eq!(consumer.len(), 1);

        producer.try_push(2).unwrap();
        assert_eq!(producer.len(), 2);

        consumer.try_pop();
        assert_eq!(consumer.len(), 1);
    }

    #[test]
    fn test_concurrent_push_pop() {
        let (mut producer, mut consumer) = SpscRingBuffer::new::<i32>(1024);

        let num_items = 10_000;

        let producer_thread = thread::spawn(move || {
            for i in 0..num_items {
                while producer.try_push(i).is_err() {
                    // Spin until space available
                    thread::yield_now();
                }
            }
        });

        let consumer_thread = thread::spawn(move || {
            let mut received = Vec::with_capacity(num_items as usize);
            while received.len() < num_items as usize {
                if let Some(value) = consumer.try_pop() {
                    received.push(value);
                } else {
                    thread::yield_now();
                }
            }
            received
        });

        producer_thread.join().unwrap();
        let received = consumer_thread.join().unwrap();

        // Verify all items received in order
        assert_eq!(received.len(), num_items as usize);
        for (i, &value) in received.iter().enumerate() {
            assert_eq!(value, i as i32);
        }
    }

    #[test]
    fn test_drop_remaining_items() {
        let counter = Rc::new(());

        {
            let (mut producer, _consumer) = SpscRingBuffer::new::<Rc<()>>(4);

            producer.try_push(Rc::clone(&counter)).unwrap();
            producer.try_push(Rc::clone(&counter)).unwrap();
            producer.try_push(Rc::clone(&counter)).unwrap();

            assert_eq!(Rc::strong_count(&counter), 4);
            // Drop producer and consumer here
        }

        // All items should be dropped
        assert_eq!(Rc::strong_count(&counter), 1);
    }

    #[test]
    fn test_partial_consumption_drop() {
        let counter = Rc::new(());

        {
            let (mut producer, mut consumer) = SpscRingBuffer::new::<Rc<()>>(4);

            producer.try_push(Rc::clone(&counter)).unwrap();
            producer.try_push(Rc::clone(&counter)).unwrap();
            producer.try_push(Rc::clone(&counter)).unwrap();

            assert_eq!(Rc::strong_count(&counter), 4);

            // Consume only one
            let _ = consumer.try_pop();
            assert_eq!(Rc::strong_count(&counter), 3);

            // Drop with 2 remaining
        }

        assert_eq!(Rc::strong_count(&counter), 1);
    }
}

#[cfg(loom)]
mod loom_tests {
    use loom::thread;

    use super::*;

    #[test]
    fn loom_concurrent_push_pop() {
        loom::model(|| {
            let (mut producer, mut consumer) = SpscRingBuffer::new::<i32>(2);

            let producer_thread = thread::spawn(move || {
                let _ = producer.try_push(1);
                let _ = producer.try_push(2);
            });

            let consumer_thread = thread::spawn(move || {
                let mut received = Vec::new();
                for _ in 0..2 {
                    if let Some(v) = consumer.try_pop() {
                        received.push(v);
                    }
                }
                received
            });

            producer_thread.join().unwrap();
            let _received = consumer_thread.join().unwrap();
        });
    }

    #[test]
    fn loom_single_item() {
        loom::model(|| {
            let (mut producer, mut consumer) = SpscRingBuffer::new::<i32>(1);

            let producer_thread = thread::spawn(move || producer.try_push(42).ok());

            let consumer_thread = thread::spawn(move || consumer.try_pop());

            let push_result = producer_thread.join().unwrap();
            let pop_result = consumer_thread.join().unwrap();

            // Either push succeeded and pop got it, or ordering caused miss
            if push_result.is_some() {
                // Push happened, pop may or may not have gotten it
                assert!(pop_result.is_none() || pop_result == Some(42));
            }
        });
    }
}
