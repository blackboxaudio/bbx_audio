//! Virtual speaker configuration for HRTF binaural rendering.
//!
//! A virtual speaker represents a point source at a specific position,
//! with associated HRIR filters for left and right ears.

use crate::graph::MAX_BLOCK_INPUTS;

/// Maximum HRIR length in samples (512 samples at 48kHz ≈ 10.7ms).
pub const MAX_HRIR_LENGTH: usize = 512;

/// Maximum number of virtual speakers supported.
pub const MAX_VIRTUAL_SPEAKERS: usize = 8;

/// A virtual speaker position with HRIR filters.
///
/// Used for HRTF-based binaural decoding. Each virtual speaker has:
/// - Spherical harmonic weights for decoding ambisonic signals
/// - Left and right ear HRIR filters
#[derive(Debug, Clone)]
pub struct VirtualSpeaker {
    /// Spherical harmonic weights for decoding B-format to this speaker.
    /// Indexed by ACN channel order.
    pub sh_weights: [f64; MAX_BLOCK_INPUTS],

    /// Left ear HRIR coefficients.
    pub left_hrir: &'static [f32],

    /// Right ear HRIR coefficients.
    pub right_hrir: &'static [f32],
}

impl VirtualSpeaker {
    /// Create a virtual speaker at the given position.
    ///
    /// # Arguments
    /// * `azimuth_deg` - Azimuth angle in degrees (0 = front, positive = left)
    /// * `elevation_deg` - Elevation angle in degrees (0 = horizon, positive = up)
    /// * `ambisonic_order` - Maximum ambisonic order to compute SH weights for
    /// * `left_hrir` - Left ear HRIR coefficients
    /// * `right_hrir` - Right ear HRIR coefficients
    pub fn new(
        azimuth_deg: f64,
        elevation_deg: f64,
        ambisonic_order: usize,
        left_hrir: &'static [f32],
        right_hrir: &'static [f32],
    ) -> Self {
        let sh_weights = compute_sh_coefficients(azimuth_deg, elevation_deg, ambisonic_order);
        Self {
            sh_weights,
            left_hrir,
            right_hrir,
        }
    }
}

/// Compute spherical harmonic coefficients for a given direction.
///
/// Uses ACN channel ordering and SN3D normalization.
fn compute_sh_coefficients(azimuth_deg: f64, elevation_deg: f64, order: usize) -> [f64; MAX_BLOCK_INPUTS] {
    let mut coeffs = [0.0; MAX_BLOCK_INPUTS];

    let az = azimuth_deg.to_radians();
    let el = elevation_deg.to_radians();

    let cos_el = el.cos();
    let sin_el = el.sin();
    let cos_az = az.cos();
    let sin_az = az.sin();

    // Order 0 (W channel)
    coeffs[0] = 1.0;

    if order >= 1 {
        // Order 1: Y, Z, X (ACN 1, 2, 3)
        coeffs[1] = cos_el * sin_az; // Y
        coeffs[2] = sin_el; // Z
        coeffs[3] = cos_el * cos_az; // X
    }

    if order >= 2 {
        // Order 2: ACN 4-8
        let cos_2az = (2.0 * az).cos();
        let sin_2az = (2.0 * az).sin();
        let sin_2el = (2.0 * el).sin();
        let cos_el_sq = cos_el * cos_el;

        coeffs[4] = 0.8660254037844386 * cos_el_sq * sin_2az; // V
        coeffs[5] = 0.8660254037844386 * sin_2el * sin_az; // T
        coeffs[6] = 0.5 * (3.0 * sin_el * sin_el - 1.0); // R
        coeffs[7] = 0.8660254037844386 * sin_2el * cos_az; // S
        coeffs[8] = 0.8660254037844386 * cos_el_sq * cos_2az; // U
    }

    if order >= 3 {
        // Order 3: ACN 9-15
        let cos_3az = (3.0 * az).cos();
        let sin_3az = (3.0 * az).sin();
        let cos_el_sq = cos_el * cos_el;
        let cos_el_cu = cos_el_sq * cos_el;
        let sin_el_sq = sin_el * sin_el;

        coeffs[9] = 0.7905694150420949 * cos_el_cu * sin_3az; // Q
        coeffs[10] = 1.9364916731037085 * cos_el_sq * sin_el * (2.0 * az).sin(); // O
        coeffs[11] = 0.6123724356957945 * cos_el * (5.0 * sin_el_sq - 1.0) * sin_az; // M
        coeffs[12] = 0.5 * sin_el * (5.0 * sin_el_sq - 3.0); // K
        coeffs[13] = 0.6123724356957945 * cos_el * (5.0 * sin_el_sq - 1.0) * cos_az; // L
        coeffs[14] = 1.9364916731037085 * cos_el_sq * sin_el * (2.0 * az).cos(); // N
        coeffs[15] = 0.7905694150420949 * cos_el_cu * cos_3az; // P
    }

    coeffs
}

/// Standard speaker layouts for binaural decoding.
pub mod layouts {
    /// FOA (First Order Ambisonics) speaker layout.
    ///
    /// 4 virtual speakers at:
    /// - Front-Left: 45° azimuth
    /// - Front-Right: -45° azimuth
    /// - Rear-Left: 135° azimuth
    /// - Rear-Right: -135° azimuth
    ///
    /// All at 0° elevation.
    pub const FOA_POSITIONS: [(f64, f64); 4] = [
        (45.0, 0.0),   // Front-Left
        (-45.0, 0.0),  // Front-Right
        (135.0, 0.0),  // Rear-Left
        (-135.0, 0.0), // Rear-Right
    ];

    /// 5.1 surround speaker positions.
    ///
    /// Standard ITU-R BS.775-1 layout:
    /// - L: 30° azimuth
    /// - R: -30° azimuth
    /// - C: 0° azimuth
    /// - LFE: 0° (not directional, but included for channel mapping)
    /// - Ls: 110° azimuth
    /// - Rs: -110° azimuth
    pub const SURROUND_51_POSITIONS: [(f64, f64); 6] = [
        (30.0, 0.0),   // L
        (-30.0, 0.0),  // R
        (0.0, 0.0),    // C
        (0.0, 0.0),    // LFE (summed to both ears equally)
        (110.0, 0.0),  // Ls
        (-110.0, 0.0), // Rs
    ];

    /// 7.1 surround speaker positions.
    ///
    /// Standard ITU-R BS.2051 layout:
    /// - L: 30° azimuth
    /// - R: -30° azimuth
    /// - C: 0° azimuth
    /// - LFE: 0° (not directional)
    /// - Ls: 90° azimuth (side left)
    /// - Rs: -90° azimuth (side right)
    /// - Lrs: 150° azimuth (rear left)
    /// - Rrs: -150° azimuth (rear right)
    pub const SURROUND_71_POSITIONS: [(f64, f64); 8] = [
        (30.0, 0.0),   // L
        (-30.0, 0.0),  // R
        (0.0, 0.0),    // C
        (0.0, 0.0),    // LFE
        (90.0, 0.0),   // Ls
        (-90.0, 0.0),  // Rs
        (150.0, 0.0),  // Lrs
        (-150.0, 0.0), // Rrs
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sh_coefficients_front() {
        // Front (azimuth=0, elevation=0) should have X positive
        let coeffs = compute_sh_coefficients(0.0, 0.0, 1);
        assert!((coeffs[0] - 1.0).abs() < 1e-10, "W should be 1.0");
        assert!(coeffs[1].abs() < 1e-10, "Y should be 0 for front");
        assert!(coeffs[2].abs() < 1e-10, "Z should be 0 for horizon");
        assert!((coeffs[3] - 1.0).abs() < 1e-10, "X should be 1.0 for front");
    }

    #[test]
    fn test_sh_coefficients_left() {
        // Left (azimuth=90, elevation=0) should have Y positive
        let coeffs = compute_sh_coefficients(90.0, 0.0, 1);
        assert!((coeffs[0] - 1.0).abs() < 1e-10, "W should be 1.0");
        assert!((coeffs[1] - 1.0).abs() < 1e-10, "Y should be 1.0 for left");
        assert!(coeffs[2].abs() < 1e-10, "Z should be 0 for horizon");
        assert!(coeffs[3].abs() < 1e-10, "X should be 0 for left");
    }

    #[test]
    fn test_sh_coefficients_above() {
        // Above (azimuth=0, elevation=90) should have Z positive
        let coeffs = compute_sh_coefficients(0.0, 90.0, 1);
        assert!((coeffs[0] - 1.0).abs() < 1e-10, "W should be 1.0");
        assert!(coeffs[1].abs() < 1e-10, "Y should be 0 for above");
        assert!((coeffs[2] - 1.0).abs() < 1e-10, "Z should be 1.0 for above");
        assert!(coeffs[3].abs() < 1e-10, "X should be 0 for above");
    }

    #[test]
    fn test_foa_layout_positions() {
        assert_eq!(layouts::FOA_POSITIONS.len(), 4);
        // Front-left should be at positive azimuth
        assert!(layouts::FOA_POSITIONS[0].0 > 0.0);
        // Front-right should be at negative azimuth
        assert!(layouts::FOA_POSITIONS[1].0 < 0.0);
    }

    #[test]
    fn test_surround_51_layout_positions() {
        assert_eq!(layouts::SURROUND_51_POSITIONS.len(), 6);
    }

    #[test]
    fn test_surround_71_layout_positions() {
        assert_eq!(layouts::SURROUND_71_POSITIONS.len(), 8);
    }
}
