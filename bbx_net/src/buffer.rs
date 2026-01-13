//! Lock-free network buffer for thread-safe message passing.
//!
//! Provides a realtime-safe channel for passing network messages between threads,
//! suitable for network input thread to audio thread communication.

use bbx_core::{
    StackVec,
    spsc::{Consumer, Producer, SpscRingBuffer},
};

use crate::message::NetMessage;

/// Maximum network events per audio buffer for stack allocation.
pub const MAX_NET_EVENTS_PER_BUFFER: usize = 64;

/// Producer side of the network buffer (used in network thread).
pub struct NetBufferProducer {
    producer: Producer<NetMessage>,
}

impl NetBufferProducer {
    /// Try to send a network message to the consumer.
    ///
    /// Returns `true` if the message was sent, `false` if the buffer was full.
    /// This method is non-blocking and safe to call from any thread.
    #[inline]
    pub fn try_send(&mut self, message: NetMessage) -> bool {
        self.producer.try_push(message).is_ok()
    }

    /// Check if the buffer is full.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.producer.is_full()
    }

    /// Returns the approximate number of messages in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.producer.len()
    }

    /// Returns `true` if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.producer.is_empty()
    }
}

/// Consumer side of the network buffer (used in audio thread).
///
/// All methods are realtime-safe and do not allocate.
pub struct NetBufferConsumer {
    consumer: Consumer<NetMessage>,
}

impl NetBufferConsumer {
    /// Pop a single network message from the buffer (realtime-safe).
    ///
    /// Returns `Some(NetMessage)` if available, `None` if empty.
    #[inline]
    pub fn try_pop(&mut self) -> Option<NetMessage> {
        self.consumer.try_pop()
    }

    /// Drain all available messages into a stack-allocated buffer.
    ///
    /// Returns a `StackVec` of messages. This is realtime-safe as it uses
    /// stack allocation with a fixed maximum capacity of `MAX_NET_EVENTS_PER_BUFFER`.
    ///
    /// If more messages are available than the stack buffer can hold,
    /// the remaining messages stay in the ring buffer for the next call.
    #[inline]
    pub fn drain_into_stack(&mut self) -> StackVec<NetMessage, MAX_NET_EVENTS_PER_BUFFER> {
        let mut buffer: StackVec<NetMessage, MAX_NET_EVENTS_PER_BUFFER> = StackVec::new();
        while !buffer.is_full() {
            match self.consumer.try_pop() {
                Some(msg) => {
                    let _ = buffer.push(msg);
                }
                None => break,
            }
        }
        buffer
    }

    /// Drain all available messages into a provided Vec.
    ///
    /// Returns the number of messages drained.
    ///
    /// **Note**: This method allocates and is NOT realtime-safe.
    /// Use `drain_into_stack()` in the audio thread instead.
    #[inline]
    pub fn drain_into(&mut self, buffer: &mut Vec<NetMessage>) -> usize {
        let mut count = 0;
        while let Some(msg) = self.consumer.try_pop() {
            buffer.push(msg);
            count += 1;
        }
        count
    }

    /// Check if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.consumer.is_empty()
    }

    /// Returns the approximate number of messages in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.consumer.len()
    }
}

/// Create a network buffer pair for thread-safe message transfer.
///
/// The `capacity` determines how many network messages can be buffered.
/// A typical value is 256-1024 messages for high-throughput scenarios.
///
/// # Examples
///
/// ```
/// use bbx_net::{NetMessage, NodeId, net_buffer};
///
/// let (mut producer, mut consumer) = net_buffer(256);
///
/// // In network thread
/// let node_id = NodeId::default();
/// let msg = NetMessage::param_change("gain", 0.5, node_id);
/// producer.try_send(msg);
///
/// // In audio thread (realtime-safe)
/// let events = consumer.drain_into_stack();
/// for event in events {
///     // Process network message
/// }
/// ```
pub fn net_buffer(capacity: usize) -> (NetBufferProducer, NetBufferConsumer) {
    let (producer, consumer) = SpscRingBuffer::new(capacity);
    (NetBufferProducer { producer }, NetBufferConsumer { consumer })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::NodeId;

    #[test]
    fn test_send_receive() {
        let (mut producer, mut consumer) = net_buffer(8);

        let node_id = NodeId::from_parts(1, 2);
        let msg = NetMessage::param_change("gain", 0.75, node_id);
        assert!(producer.try_send(msg));

        let received = consumer.try_pop().expect("should have message");
        assert!((received.value - 0.75).abs() < f32::EPSILON);
        assert!(consumer.try_pop().is_none());
    }

    #[test]
    fn test_buffer_overflow() {
        let (mut producer, _consumer) = net_buffer(2);

        let node_id = NodeId::from_parts(1, 2);
        let msg = NetMessage::param_change("test", 0.5, node_id);

        assert!(producer.try_send(msg));
        assert!(producer.try_send(msg));
        assert!(!producer.try_send(msg));
        assert!(producer.is_full());
    }

    #[test]
    fn test_drain_into_stack() {
        let (mut producer, mut consumer) = net_buffer(8);

        let node_id = NodeId::from_parts(1, 2);
        producer.try_send(NetMessage::param_change("p1", 0.1, node_id));
        producer.try_send(NetMessage::param_change("p2", 0.2, node_id));
        producer.try_send(NetMessage::param_change("p3", 0.3, node_id));

        let stack = consumer.drain_into_stack();

        assert_eq!(stack.len(), 3);
        assert!((stack[0].value - 0.1).abs() < f32::EPSILON);
        assert!((stack[1].value - 0.2).abs() < f32::EPSILON);
        assert!((stack[2].value - 0.3).abs() < f32::EPSILON);
        assert!(consumer.is_empty());
    }

    #[test]
    fn test_drain_into_vec() {
        let (mut producer, mut consumer) = net_buffer(8);

        let node_id = NodeId::from_parts(3, 4);
        producer.try_send(NetMessage::trigger("a", node_id));
        producer.try_send(NetMessage::trigger("b", node_id));

        let mut buffer = Vec::new();
        let count = consumer.drain_into(&mut buffer);

        assert_eq!(count, 2);
        assert_eq!(buffer.len(), 2);
        assert!(consumer.is_empty());
    }

    #[test]
    fn test_is_empty() {
        let (mut producer, mut consumer) = net_buffer(4);

        assert!(consumer.is_empty());

        let node_id = NodeId::from_parts(5, 6);
        producer.try_send(NetMessage::param_change("x", 0.0, node_id));
        assert!(!consumer.is_empty());

        consumer.try_pop();
        assert!(consumer.is_empty());
    }

    #[test]
    fn test_len() {
        let (mut producer, consumer) = net_buffer(8);

        assert_eq!(producer.len(), 0);
        assert_eq!(consumer.len(), 0);

        let node_id = NodeId::from_parts(7, 8);
        producer.try_send(NetMessage::param_change("a", 0.0, node_id));
        producer.try_send(NetMessage::param_change("b", 0.0, node_id));

        assert_eq!(producer.len(), 2);
        assert_eq!(consumer.len(), 2);
    }

    #[test]
    fn test_drain_stack_overflow_preserves_remaining() {
        let (mut producer, mut consumer) = net_buffer(128);

        let node_id = NodeId::from_parts(9, 10);

        for i in 0..100 {
            producer.try_send(NetMessage::param_change("x", i as f32, node_id));
        }

        let stack = consumer.drain_into_stack();
        assert_eq!(stack.len(), MAX_NET_EVENTS_PER_BUFFER);

        let remaining = 100 - MAX_NET_EVENTS_PER_BUFFER;
        assert_eq!(consumer.len(), remaining);
    }
}
