//! Virtual speaker configuration for HRTF binaural rendering.
//!
//! A virtual speaker represents a point source at a specific position,
//! with associated HRIR filters for left and right ears.

use crate::block::MAX_BLOCK_INPUTS;

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
        let sh_weights = compute_sh_coefficients_max_re(azimuth_deg, elevation_deg, ambisonic_order);
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

/// Compute max-rE weights for the given ambisonic order.
///
/// max-rE weighting improves perceived localization by concentrating
/// energy toward the intended source direction. This is particularly
/// important when decoding to a finite number of virtual speakers.
fn compute_max_re_weights(order: usize) -> [f64; 4] {
    // max-rE angle: 137.9 degrees in radians
    const ALPHA: f64 = 2.406184877014388;

    let mut weights = [1.0; 4];
    let divisor = 2.0 * (order + 1) as f64;

    for (l, weight) in weights.iter_mut().enumerate().take(order.min(3) + 1).skip(1) {
        *weight = (ALPHA / divisor).cos().powi(l as i32);
    }

    weights
}

/// Compute spherical harmonic coefficients with max-rE weighting.
///
/// This variant applies max-rE weights to improve perceived localization
/// when decoding ambisonics to a finite speaker array.
fn compute_sh_coefficients_max_re(azimuth_deg: f64, elevation_deg: f64, order: usize) -> [f64; MAX_BLOCK_INPUTS] {
    let mut coeffs = compute_sh_coefficients(azimuth_deg, elevation_deg, order);
    let weights = compute_max_re_weights(order);

    // Apply weights by order (ACN ordering: order l occupies indices l² to (l+1)²-1)
    for (l, &weight) in weights.iter().enumerate().take(order.min(3) + 1) {
        let start = l * l;
        let end = (l + 1) * (l + 1);
        for coeff in coeffs.iter_mut().take(end).skip(start) {
            *coeff *= weight;
        }
    }

    coeffs
}

/// Standard speaker layouts for binaural decoding.
pub mod layouts {
    /// FOA (First Order Ambisonics) speaker layout.
    ///
    /// 8 virtual speakers at all available HRIR positions for optimal
    /// spatial coverage:
    /// - Front: 0° azimuth
    /// - Front-Left: 45° azimuth
    /// - Left: 90° azimuth
    /// - Rear-Left: 135° azimuth
    /// - Rear: 180° azimuth
    /// - Rear-Right: -135° azimuth
    /// - Right: -90° azimuth
    /// - Front-Right: -45° azimuth
    ///
    /// All at 0° elevation.
    pub const FOA_POSITIONS: [(f64, f64); 8] = [
        (0.0, 0.0),    // Front
        (45.0, 0.0),   // Front-Left
        (90.0, 0.0),   // Left
        (135.0, 0.0),  // Rear-Left
        (180.0, 0.0),  // Rear
        (-135.0, 0.0), // Rear-Right
        (-90.0, 0.0),  // Right
        (-45.0, 0.0),  // Front-Right
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
        assert_eq!(layouts::FOA_POSITIONS.len(), 8);
        // First position should be front (0 degrees)
        assert!((layouts::FOA_POSITIONS[0].0).abs() < 1e-10);
        // Should have positions at cardinal and diagonal directions
        let azimuths: Vec<f64> = layouts::FOA_POSITIONS.iter().map(|(az, _)| *az).collect();
        assert!(azimuths.contains(&0.0));
        assert!(azimuths.contains(&90.0));
        assert!(azimuths.contains(&180.0));
        assert!(azimuths.contains(&-90.0));
    }

    #[test]
    fn test_max_re_weights_foa() {
        let weights = super::compute_max_re_weights(1);
        // Order 0 weight should be 1.0
        assert!((weights[0] - 1.0).abs() < 1e-10);
        // Order 1 weight: cos(137.9°/4) ≈ 0.824
        assert!((weights[1] - 0.824).abs() < 0.01);
    }

    #[test]
    fn test_sh_coefficients_max_re_reduces_order1() {
        // max-rE should reduce order 1 coefficients relative to basic SH
        let basic = super::compute_sh_coefficients(45.0, 0.0, 1);
        let max_re = super::compute_sh_coefficients_max_re(45.0, 0.0, 1);

        // Order 0 (W) should be unchanged
        assert!((basic[0] - max_re[0]).abs() < 1e-10);

        // Order 1 coefficients should be scaled down
        for i in 1..4 {
            if basic[i].abs() > 1e-10 {
                assert!(max_re[i].abs() < basic[i].abs());
            }
        }
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
