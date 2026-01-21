//! SAI/I2S audio interface.
//!
//! This module will provide the audio interface for Daisy hardware,
//! handling SAI (Serial Audio Interface) configuration and DMA transfers.
//!
//! # Implementation Notes (Phase 4)
//!
//! The Daisy Seed uses the STM32H750's SAI peripheral in I2S mode:
//!
//! - SAI1 Block A: Transmit (to codec DAC)
//! - SAI1 Block B: Receive (from codec ADC)
//! - DMA2 for zero-copy circular buffer transfers
//!
//! The audio callback pattern will look like:
//!
//! ```ignore
//! fn audio_callback(input: &FrameBuffer<N>, output: &mut FrameBuffer<N>) {
//!     // Process audio here
//! }
//! ```

// Placeholder for Phase 4 implementation
