//! ADC abstraction.
//!
//! This module will provide convenient ADC access for reading
//! potentiometers, CV inputs, and other analog signals.
//!
//! # Implementation Notes (Phase 4)
//!
//! The STM32H750 has three ADCs that can be used in various configurations:
//!
//! - Single-shot conversion for occasional reads
//! - DMA-based continuous conversion for control-rate sampling
//! - Injected conversion for high-priority reads
//!
//! For audio control signals (knobs, CV), DMA-based continuous conversion
//! at control rate (typically 1kHz) is recommended to avoid blocking
//! the audio callback.
//!
//! # CV Input Ranges
//!
//! Different Daisy products have different CV input ranges:
//!
//! - Seed: 0V to 3.3V (direct ADC input)
//! - Patch SM: -5V to +5V (with analog conditioning)

// Placeholder for Phase 4 implementation
