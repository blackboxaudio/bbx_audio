//! SAI/I2S audio interface with DMA transfers.
//!
//! This module provides the audio interface for Daisy hardware using the
//! STM32H750's SAI peripheral in I2S master mode with circular DMA.
//!
//! # Architecture
//!
//! - SAI1 Block A: Transmit (to codec DAC)
//! - SAI1 Block B: Receive (from codec ADC)
//! - DMA1: Circular buffer transfers with half/complete interrupts
//!
//! # Usage
//!
//! ```ignore
//! use bbx_daisy::{audio, FrameBuffer};
//!
//! fn audio_callback(input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
//!     // Process audio: copy input to output with some processing
//!     for i in 0..BLOCK_SIZE {
//!         let [left, right] = *input.frame(i);
//!         output.set_frame(i, left * 0.5, right * 0.5);
//!     }
//! }
//!
//! fn main() {
//!     // ... initialize hardware ...
//!     audio::set_callback(audio_callback);
//!     audio::start();
//! }
//! ```

use core::{
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{buffer::FrameBuffer, clock::SampleRate};

/// Default block size for audio processing (32 samples at ~48kHz = 0.67ms latency).
pub const BLOCK_SIZE: usize = 32;

/// DMA buffer size (double-buffered, so 2x block size).
const DMA_BUFFER_SIZE: usize = BLOCK_SIZE * 2;

/// Audio callback function type.
///
/// Called from the DMA interrupt with input samples and output buffer to fill.
/// Must complete within the buffer period (~0.67ms at 48kHz/32 samples).
pub type AudioCallback = fn(input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>);

/// Default passthrough callback (copies input to output).
fn default_callback(input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
    for i in 0..BLOCK_SIZE {
        let frame = input.frame(i);
        output.set_frame(i, frame[0], frame[1]);
    }
}

// ============================================================================
// Global State (interrupt-safe)
// ============================================================================

/// Global audio callback function pointer.
static mut AUDIO_CALLBACK: AudioCallback = default_callback;

/// Flag indicating audio is running.
static AUDIO_RUNNING: AtomicBool = AtomicBool::new(false);

/// DMA transmit buffer (placed in DMA-accessible SRAM3).
#[unsafe(link_section = ".sram3")]
static mut TX_BUFFER: MaybeUninit<[[i32; 2]; DMA_BUFFER_SIZE]> = MaybeUninit::uninit();

/// DMA receive buffer (placed in DMA-accessible SRAM3).
#[unsafe(link_section = ".sram3")]
static mut RX_BUFFER: MaybeUninit<[[i32; 2]; DMA_BUFFER_SIZE]> = MaybeUninit::uninit();

// ============================================================================
// Public API
// ============================================================================

/// Set the audio callback function.
///
/// # Safety
///
/// Must be called before `start()` or while audio is stopped.
/// The callback must be realtime-safe (no allocations, no blocking).
pub fn set_callback(callback: AudioCallback) {
    // Safety: Only safe to call when audio is not running
    if !AUDIO_RUNNING.load(Ordering::SeqCst) {
        unsafe {
            AUDIO_CALLBACK = callback;
        }
    }
}

/// Check if audio is currently running.
pub fn is_running() -> bool {
    AUDIO_RUNNING.load(Ordering::SeqCst)
}

/// Audio interface configuration.
pub struct AudioConfig {
    /// Sample rate (48kHz or 96kHz).
    pub sample_rate: SampleRate,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: SampleRate::Rate48000,
        }
    }
}

/// Audio interface handle.
///
/// Manages SAI and DMA peripherals for audio I/O.
pub struct AudioInterface {
    config: AudioConfig,
}

impl AudioInterface {
    /// Create a new audio interface with the given configuration.
    pub fn new(config: AudioConfig) -> Self {
        Self { config }
    }

    /// Get the configured sample rate.
    pub fn sample_rate(&self) -> SampleRate {
        self.config.sample_rate
    }

    /// Get the block size in samples.
    pub const fn block_size(&self) -> usize {
        BLOCK_SIZE
    }

    /// Initialize the audio interface.
    ///
    /// This configures SAI1 in I2S master mode and sets up DMA transfers.
    /// Call this after clock configuration and before `start()`.
    pub fn init(&mut self) {
        // Initialize DMA buffers to zero using raw pointers (Rust 2024 compatible)
        unsafe {
            let tx_ptr = core::ptr::addr_of_mut!(TX_BUFFER);
            let rx_ptr = core::ptr::addr_of_mut!(RX_BUFFER);
            (*tx_ptr).write([[0i32; 2]; DMA_BUFFER_SIZE]);
            (*rx_ptr).write([[0i32; 2]; DMA_BUFFER_SIZE]);
        }

        // SAI and DMA configuration would go here
        // For now, this is a placeholder that shows the structure

        // In a full implementation:
        // 1. Configure SAI1 Block A (TX) and Block B (RX) for I2S
        // 2. Configure DMA1 Stream 0 for SAI1_A (TX)
        // 3. Configure DMA1 Stream 1 for SAI1_B (RX)
        // 4. Enable half-transfer and transfer-complete interrupts
    }

    /// Start audio processing.
    ///
    /// Begins DMA transfers and enables interrupts.
    pub fn start(&mut self) {
        if AUDIO_RUNNING.swap(true, Ordering::SeqCst) {
            return; // Already running
        }

        // Enable DMA and SAI
        // In a full implementation:
        // 1. Enable DMA streams
        // 2. Enable SAI blocks
    }

    /// Stop audio processing.
    ///
    /// Stops DMA transfers and disables interrupts.
    pub fn stop(&mut self) {
        if !AUDIO_RUNNING.swap(false, Ordering::SeqCst) {
            return; // Already stopped
        }

        // Disable SAI and DMA
        // In a full implementation:
        // 1. Disable SAI blocks
        // 2. Disable DMA streams
    }
}

// ============================================================================
// DMA Interrupt Handler
// ============================================================================

/// Process audio at the given buffer half (0 = first half, 1 = second half).
///
/// Called from DMA interrupt on half-transfer and transfer-complete events.
///
/// # Safety
///
/// Must only be called from the DMA interrupt handler.
#[inline(always)]
unsafe fn process_audio_buffer(buffer_half: usize) {
    unsafe {
        let offset = buffer_half * BLOCK_SIZE;

        // Get raw pointers to the buffers (Rust 2024 compatible)
        let tx_ptr = core::ptr::addr_of_mut!(TX_BUFFER);
        let rx_ptr = core::ptr::addr_of!(RX_BUFFER);

        let tx_buf = (*tx_ptr).assume_init_mut();
        let rx_buf = (*rx_ptr).assume_init_ref();

        // Convert DMA samples (i32) to f32 for processing
        let mut input: FrameBuffer<BLOCK_SIZE> = FrameBuffer::new();
        let mut output: FrameBuffer<BLOCK_SIZE> = FrameBuffer::new();

        // Deinterleave and convert RX buffer to FrameBuffer
        for i in 0..BLOCK_SIZE {
            let [left_i32, right_i32] = rx_buf[offset + i];
            let left = i32_to_f32(left_i32);
            let right = i32_to_f32(right_i32);
            input.set_frame(i, left, right);
        }

        // Call user callback
        let callback = core::ptr::addr_of!(AUDIO_CALLBACK);
        (*callback)(&input, &mut output);

        // Convert and interleave output FrameBuffer to TX buffer
        for i in 0..BLOCK_SIZE {
            let [left, right] = *output.frame(i);
            tx_buf[offset + i][0] = f32_to_i32(left);
            tx_buf[offset + i][1] = f32_to_i32(right);
        }
    }
}

/// Convert 24-bit I2S sample (in i32) to f32 [-1.0, 1.0].
#[inline(always)]
fn i32_to_f32(sample: i32) -> f32 {
    // I2S 24-bit samples are left-justified in 32-bit words
    // Shift right by 8 to get 24-bit value, then normalize
    const SCALE: f32 = 1.0 / 8388608.0; // 1 / 2^23
    (sample >> 8) as f32 * SCALE
}

/// Convert f32 [-1.0, 1.0] to 24-bit I2S sample (in i32).
#[inline(always)]
fn f32_to_i32(sample: f32) -> i32 {
    // Clamp to valid range, scale to 24-bit, left-justify in 32-bit
    const SCALE: f32 = 8388607.0; // 2^23 - 1
    let clamped = sample.clamp(-1.0, 1.0);
    ((clamped * SCALE) as i32) << 8
}

// ============================================================================
// Interrupt Handlers (to be connected by the application)
// ============================================================================

/// DMA1 Stream 0 interrupt handler (SAI1 TX half/complete).
///
/// Call this from the DMA1_STR0 interrupt vector.
///
/// # Safety
///
/// Must only be called from the DMA interrupt context.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dma1_str0_handler(half_transfer: bool) {
    if !AUDIO_RUNNING.load(Ordering::Relaxed) {
        return;
    }

    let buffer_half = if half_transfer { 0 } else { 1 };
    unsafe { process_audio_buffer(buffer_half) };
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Create an audio interface with default configuration (48kHz).
pub fn default_audio() -> AudioInterface {
    AudioInterface::new(AudioConfig::default())
}

/// Create an audio interface with specified sample rate.
pub fn audio_with_rate(sample_rate: SampleRate) -> AudioInterface {
    AudioInterface::new(AudioConfig { sample_rate })
}
