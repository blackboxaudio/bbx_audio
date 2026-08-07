//! Peripheral abstractions for Daisy hardware.
//!
//! This module provides high-level abstractions over STM32H750 peripherals
//! commonly used on Daisy boards.
//!
//! ## GPIO
//!
//! - [`gpio::Led`] - LED control wrapper
//! - [`gpio::Button`] - Debounced button input
//! - [`gpio::GateIn`] / [`gpio::GateOut`] - Gate I/O for Eurorack
//!
//! ## ADC
//!
//! - [`adc::Knob`] - Potentiometer with smoothing
//! - [`adc::CvInput`] - CV input with bipolar/unipolar support
//!
//! ## Encoders
//!
//! - [`encoder::Encoder`] - Quadrature encoder
//! - [`encoder::EncoderWithButton`] - Encoder with integrated push button

pub mod adc;
pub mod encoder;
pub mod gpio;

pub use adc::{CvInput, CvRange, Knob};
pub use encoder::{Direction, Encoder, EncoderWithButton, VelocityEncoder};
pub use gpio::{Button, GateIn, GateOut, Led};
