//! Rotary encoder abstraction.
//!
//! This module will provide support for quadrature rotary encoders
//! with push buttons, commonly found on Daisy Pod and similar boards.
//!
//! # Implementation Notes (Phase 4)
//!
//! Encoder reading approaches:
//!
//! - Timer-based: Use STM32 timer in encoder mode (hardware quadrature decoding)
//! - Interrupt-based: GPIO interrupts on A/B transitions
//! - Polling-based: Read A/B pins at control rate
//!
//! Timer-based is most robust but uses timer resources.
//! Interrupt-based works well for low-speed UI encoders.
//!
//! # Features
//!
//! - Direction detection (CW/CCW)
//! - Velocity/acceleration sensing
//! - Button press with debouncing
//! - Long-press detection

// Placeholder for Phase 4 implementation
