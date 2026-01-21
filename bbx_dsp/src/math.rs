//! Math function compatibility for std and no_std (libm) environments.
//!
//! In std builds, this re-exports standard library methods.
//! In no_std builds with libm, this provides equivalent functionality.

/// Sine function
#[inline]
pub fn sin_f64(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.sin()
    }
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    {
        libm::sin(x)
    }
}

/// Cosine function
#[inline]
pub fn cos_f64(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.cos()
    }
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    {
        libm::cos(x)
    }
}

/// Sine function (f32)
#[inline]
pub fn sin_f32(x: f32) -> f32 {
    #[cfg(feature = "std")]
    {
        x.sin()
    }
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    {
        libm::sinf(x)
    }
}

/// Cosine function (f32)
#[inline]
pub fn cos_f32(x: f32) -> f32 {
    #[cfg(feature = "std")]
    {
        x.cos()
    }
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    {
        libm::cosf(x)
    }
}

/// Power function
#[inline]
pub fn powf_f64(base: f64, exp: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        base.powf(exp)
    }
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    {
        libm::pow(base, exp)
    }
}

/// Power function (f32)
#[inline]
pub fn powf_f32(base: f32, exp: f32) -> f32 {
    #[cfg(feature = "std")]
    {
        base.powf(exp)
    }
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    {
        libm::powf(base, exp)
    }
}

/// Natural logarithm
#[inline]
pub fn ln_f64(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.ln()
    }
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    {
        libm::log(x)
    }
}

/// Exponential function
#[inline]
pub fn exp_f64(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.exp()
    }
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    {
        libm::exp(x)
    }
}

/// Floor function
#[inline]
pub fn floor_f64(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.floor()
    }
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    {
        libm::floor(x)
    }
}

/// Tangent hyperbolic function
#[inline]
pub fn tanh_f64(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.tanh()
    }
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    {
        libm::tanh(x)
    }
}

/// Square root function
#[inline]
pub fn sqrt_f64(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.sqrt()
    }
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    {
        libm::sqrt(x)
    }
}

/// Converts radians to degrees
#[inline]
pub fn to_radians_f64(deg: f64) -> f64 {
    deg * core::f64::consts::PI / 180.0
}
