//! # BBX Core
//!
//! This crate contains the core mechanisms and utilities
//! used throughout this workspace.

pub mod denormal;
pub mod error;
pub mod random;

pub use denormal::{flush_denormal_f32, flush_denormal_f64};
pub use error::{BbxError, Result};
