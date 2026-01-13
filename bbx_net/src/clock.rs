//! Clock synchronization for distributed audio timing.
//!
//! Provides `SyncedTimestamp` for representing synchronized time across nodes
//! and `ClockSync` for managing clock synchronization state.

use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

/// A synchronized timestamp in microseconds since server start.
///
/// Used for scheduling events across distributed nodes with sample-accurate
/// timing within audio buffers.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct SyncedTimestamp(pub u64);

impl SyncedTimestamp {
    /// Create a timestamp from microseconds.
    pub const fn from_micros(micros: u64) -> Self {
        Self(micros)
    }

    /// Get the timestamp value in microseconds.
    pub const fn as_micros(&self) -> u64 {
        self.0
    }

    /// Convert to sample offset within a buffer.
    ///
    /// Returns `Some(offset)` if the timestamp falls within the current buffer,
    /// `None` if it's in a future buffer. Events in the past return offset 0.
    ///
    /// # Arguments
    ///
    /// * `buffer_start_time` - Timestamp at the start of the current buffer
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `buffer_size` - Number of samples in the buffer
    pub fn to_sample_offset(
        &self,
        buffer_start_time: SyncedTimestamp,
        sample_rate: f64,
        buffer_size: usize,
    ) -> Option<u32> {
        if self.0 < buffer_start_time.0 {
            return Some(0);
        }

        let delta_micros = self.0 - buffer_start_time.0;
        let delta_seconds = delta_micros as f64 / 1_000_000.0;
        let sample_offset = (delta_seconds * sample_rate) as usize;

        if sample_offset < buffer_size {
            Some(sample_offset as u32)
        } else {
            None
        }
    }

    /// Calculate the difference between two timestamps in microseconds.
    pub fn delta(&self, other: SyncedTimestamp) -> i64 {
        self.0 as i64 - other.0 as i64
    }
}

/// Clock synchronization state for a node.
///
/// Manages the mapping between local time and synchronized network time.
/// Thread-safe for reading the current time from the audio thread.
pub struct ClockSync {
    start_instant: Instant,
    current_time: AtomicU64,
}

impl ClockSync {
    /// Create a new clock synchronization instance.
    pub fn new() -> Self {
        Self {
            start_instant: Instant::now(),
            current_time: AtomicU64::new(0),
        }
    }

    /// Get the current synchronized timestamp.
    ///
    /// This reads the system clock and converts to microseconds since start.
    ///
    /// # Realtime Safety
    ///
    /// This method is NOT realtime-safe as it calls `Instant::elapsed()` which
    /// may invoke a system call. For audio thread use, call [`tick()`](Self::tick)
    /// from a non-audio thread and use [`cached_now()`](Self::cached_now) from
    /// the audio thread.
    #[inline]
    pub fn now(&self) -> SyncedTimestamp {
        let elapsed = self.start_instant.elapsed();
        SyncedTimestamp(elapsed.as_micros() as u64)
    }

    /// Update the cached current time.
    ///
    /// Call this periodically (e.g., at the start of each audio buffer)
    /// to update the cached time value.
    ///
    /// # Realtime Safety
    ///
    /// This method is NOT realtime-safe as it calls [`now()`](Self::now)
    /// internally. Call this from your main thread or audio device callback
    /// thread, then use [`cached_now()`](Self::cached_now) from the audio
    /// processing code.
    pub fn tick(&self) {
        let now = self.now();
        self.current_time.store(now.0, Ordering::Relaxed);
    }

    /// Get the cached current time.
    ///
    /// Faster than `now()` as it avoids a system call, but may be slightly stale.
    /// Use for non-critical timing within the audio thread.
    #[inline]
    pub fn cached_now(&self) -> SyncedTimestamp {
        SyncedTimestamp(self.current_time.load(Ordering::Relaxed))
    }

    /// Get the time elapsed since clock creation in microseconds.
    pub fn elapsed_micros(&self) -> u64 {
        self.start_instant.elapsed().as_micros() as u64
    }

    /// Calculate client clock offset based on ping/pong exchange.
    ///
    /// Uses NTP-style offset calculation to determine the difference between
    /// client and server clocks.
    ///
    /// # Arguments
    ///
    /// * `client_send_time` - Client timestamp when ping was sent
    /// * `server_receive_time` - Server timestamp when ping was received
    /// * `server_send_time` - Server timestamp when pong was sent
    /// * `client_receive_time` - Client timestamp when pong was received
    ///
    /// # Returns
    ///
    /// Clock offset in microseconds (positive = client ahead, negative = client behind)
    pub fn calculate_offset(
        client_send_time: u64,
        server_receive_time: u64,
        server_send_time: u64,
        client_receive_time: u64,
    ) -> i64 {
        let t1 = client_send_time as i64;
        let t2 = server_receive_time as i64;
        let t3 = server_send_time as i64;
        let t4 = client_receive_time as i64;

        ((t2 - t1) + (t3 - t4)) / 2
    }
}

impl Default for ClockSync {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;

    #[test]
    fn test_synced_timestamp_default() {
        let ts = SyncedTimestamp::default();
        assert_eq!(ts.as_micros(), 0);
    }

    #[test]
    fn test_synced_timestamp_from_micros() {
        let ts = SyncedTimestamp::from_micros(1_000_000);
        assert_eq!(ts.as_micros(), 1_000_000);
    }

    #[test]
    fn test_synced_timestamp_ordering() {
        let ts1 = SyncedTimestamp::from_micros(100);
        let ts2 = SyncedTimestamp::from_micros(200);
        assert!(ts1 < ts2);
    }

    #[test]
    fn test_synced_timestamp_delta() {
        let ts1 = SyncedTimestamp::from_micros(1000);
        let ts2 = SyncedTimestamp::from_micros(500);
        assert_eq!(ts1.delta(ts2), 500);
        assert_eq!(ts2.delta(ts1), -500);
    }

    #[test]
    fn test_to_sample_offset_in_buffer() {
        let buffer_start = SyncedTimestamp::from_micros(0);
        let event_time = SyncedTimestamp::from_micros(500);

        let offset = event_time.to_sample_offset(buffer_start, 44100.0, 512).unwrap();

        let expected = (0.0005 * 44100.0) as u32;
        assert_eq!(offset, expected);
    }

    #[test]
    fn test_to_sample_offset_past_event() {
        let buffer_start = SyncedTimestamp::from_micros(1000);
        let event_time = SyncedTimestamp::from_micros(500);

        let offset = event_time.to_sample_offset(buffer_start, 44100.0, 512).unwrap();
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_to_sample_offset_future_buffer() {
        let buffer_start = SyncedTimestamp::from_micros(0);
        let event_time = SyncedTimestamp::from_micros(1_000_000);

        let offset = event_time.to_sample_offset(buffer_start, 44100.0, 512);
        assert!(offset.is_none());
    }

    #[test]
    fn test_clock_sync_now_increases() {
        let clock = ClockSync::new();
        let t1 = clock.now();
        thread::sleep(Duration::from_millis(10));
        let t2 = clock.now();
        assert!(t2 > t1);
    }

    #[test]
    fn test_clock_sync_tick_updates_cached() {
        let clock = ClockSync::new();
        clock.tick();
        let cached1 = clock.cached_now();
        thread::sleep(Duration::from_millis(10));
        clock.tick();
        let cached2 = clock.cached_now();
        assert!(cached2 > cached1);
    }

    #[test]
    fn test_calculate_offset_symmetric() {
        let offset = ClockSync::calculate_offset(100, 200, 200, 300);
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_calculate_offset_client_ahead() {
        let offset = ClockSync::calculate_offset(100, 50, 50, 100);
        assert!(offset < 0);
    }

    #[test]
    fn test_calculate_offset_client_behind() {
        let offset = ClockSync::calculate_offset(100, 250, 250, 300);
        assert!(offset > 0);
    }
}
