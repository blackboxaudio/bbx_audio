//! Head-Related Impulse Response (HRIR) data for binaural decoding.
//!
//! This module provides HRIR coefficients for various speaker directions.
//! Currently uses placeholder/approximate HRIRs that provide basic ILD and ITD cues.
//!
//! # Future Work
//! Replace with measured HRIRs from MIT KEMAR or SADIE II database:
//! - MIT KEMAR: <https://sound.media.mit.edu/resources/KEMAR.html> (44.1kHz, free with attribution)
//! - SADIE II: <https://www.york.ac.uk/sadie-project/> (48kHz, Apache 2.0)

/// Length of the HRIRs in samples.
pub const HRIR_LENGTH: usize = 128;

/// HRIR pair (left and right ear).
pub struct HrirPair {
    pub left: &'static [f32; HRIR_LENGTH],
    pub right: &'static [f32; HRIR_LENGTH],
}

/// Get HRIR pair for a given azimuth (elevation assumed 0).
///
/// Returns the closest matching HRIR from the available set.
pub fn get_hrir_for_azimuth(azimuth_deg: f64) -> HrirPair {
    // Normalize azimuth to -180..180
    let az = ((azimuth_deg + 180.0) % 360.0) - 180.0;

    // Select closest HRIR based on azimuth
    if (-22.5..22.5).contains(&az) {
        HrirPair {
            left: &HRIR_FRONT_LEFT,
            right: &HRIR_FRONT_RIGHT,
        }
    } else if (22.5..67.5).contains(&az) {
        HrirPair {
            left: &HRIR_FRONT_LEFT_45_LEFT,
            right: &HRIR_FRONT_LEFT_45_RIGHT,
        }
    } else if (67.5..112.5).contains(&az) {
        HrirPair {
            left: &HRIR_LEFT_90_LEFT,
            right: &HRIR_LEFT_90_RIGHT,
        }
    } else if !(-157.5..112.5).contains(&az) {
        HrirPair {
            left: &HRIR_REAR_LEFT,
            right: &HRIR_REAR_RIGHT,
        }
    } else if (-67.5..-22.5).contains(&az) {
        HrirPair {
            left: &HRIR_FRONT_RIGHT_45_LEFT,
            right: &HRIR_FRONT_RIGHT_45_RIGHT,
        }
    } else if (-112.5..-67.5).contains(&az) {
        HrirPair {
            left: &HRIR_RIGHT_90_LEFT,
            right: &HRIR_RIGHT_90_RIGHT,
        }
    } else if (-157.5..-112.5).contains(&az) {
        HrirPair {
            left: &HRIR_REAR_RIGHT_LEFT,
            right: &HRIR_REAR_RIGHT_RIGHT,
        }
    } else {
        HrirPair {
            left: &HRIR_REAR_LEFT_LEFT,
            right: &HRIR_REAR_LEFT_RIGHT,
        }
    }
}

// Placeholder HRIR data
// These are approximate impulse responses that provide basic ILD/ITD cues.
// Structure: short delay + main impulse + decay

/// Generate a basic HRIR with delay and decay.
const fn generate_placeholder_hrir(
    delay_samples: usize,
    peak_amplitude: f32,
    contralateral: bool,
) -> [f32; HRIR_LENGTH] {
    let mut hrir = [0.0f32; HRIR_LENGTH];
    let mut i = 0;

    // Skip delay samples
    while i < delay_samples && i < HRIR_LENGTH {
        i += 1;
    }

    // Main impulse and decay
    if i < HRIR_LENGTH {
        // Use a simpler decay pattern for const fn
        let mut amplitude = peak_amplitude;
        let decay = if contralateral { 0.85 } else { 0.88 };

        while i < HRIR_LENGTH && amplitude.abs() > 0.001 {
            hrir[i] = amplitude;
            amplitude *= decay;
            if (i - delay_samples) % 2 == 1 {
                amplitude = -amplitude * 0.7;
            }
            i += 1;
        }
    }

    hrir
}

// Front (0°) - equal timing, balanced amplitude
static HRIR_FRONT_LEFT: [f32; HRIR_LENGTH] = generate_placeholder_hrir(4, 0.5, false);
static HRIR_FRONT_RIGHT: [f32; HRIR_LENGTH] = generate_placeholder_hrir(4, 0.5, false);

// Front-Left (45°) - left ear leads, louder in left
static HRIR_FRONT_LEFT_45_LEFT: [f32; HRIR_LENGTH] = generate_placeholder_hrir(3, 0.6, false);
static HRIR_FRONT_LEFT_45_RIGHT: [f32; HRIR_LENGTH] = generate_placeholder_hrir(6, 0.35, true);

// Left (90°) - left ear leads significantly, much louder in left
static HRIR_LEFT_90_LEFT: [f32; HRIR_LENGTH] = generate_placeholder_hrir(2, 0.7, false);
static HRIR_LEFT_90_RIGHT: [f32; HRIR_LENGTH] = generate_placeholder_hrir(9, 0.25, true);

// Rear-Left (135°) - left ear leads, moderate ILD
static HRIR_REAR_LEFT_LEFT: [f32; HRIR_LENGTH] = generate_placeholder_hrir(3, 0.55, false);
static HRIR_REAR_LEFT_RIGHT: [f32; HRIR_LENGTH] = generate_placeholder_hrir(7, 0.3, true);

// Rear (180°) - equal timing, balanced amplitude, more diffuse
static HRIR_REAR_LEFT: [f32; HRIR_LENGTH] = generate_placeholder_hrir(5, 0.45, false);
static HRIR_REAR_RIGHT: [f32; HRIR_LENGTH] = generate_placeholder_hrir(5, 0.45, false);

// Front-Right (-45°) - right ear leads, louder in right
static HRIR_FRONT_RIGHT_45_LEFT: [f32; HRIR_LENGTH] = generate_placeholder_hrir(6, 0.35, true);
static HRIR_FRONT_RIGHT_45_RIGHT: [f32; HRIR_LENGTH] = generate_placeholder_hrir(3, 0.6, false);

// Right (-90°) - right ear leads significantly, much louder in right
static HRIR_RIGHT_90_LEFT: [f32; HRIR_LENGTH] = generate_placeholder_hrir(9, 0.25, true);
static HRIR_RIGHT_90_RIGHT: [f32; HRIR_LENGTH] = generate_placeholder_hrir(2, 0.7, false);

// Rear-Right (-135°) - right ear leads, moderate ILD
static HRIR_REAR_RIGHT_LEFT: [f32; HRIR_LENGTH] = generate_placeholder_hrir(7, 0.3, true);
static HRIR_REAR_RIGHT_RIGHT: [f32; HRIR_LENGTH] = generate_placeholder_hrir(3, 0.55, false);

#[cfg(test)]
mod tests {
    use super::{super::virtual_speaker::MAX_HRIR_LENGTH, *};

    #[test]
    fn test_hrir_length() {
        assert!(HRIR_LENGTH <= MAX_HRIR_LENGTH);
    }

    #[test]
    fn test_get_hrir_front() {
        let hrir = get_hrir_for_azimuth(0.0);
        // Front should have similar amplitudes for both ears
        let left_energy: f32 = hrir.left.iter().map(|x| x * x).sum();
        let right_energy: f32 = hrir.right.iter().map(|x| x * x).sum();
        let ratio = left_energy / right_energy;
        assert!(ratio > 0.8 && ratio < 1.2, "Front should be balanced, ratio={}", ratio);
    }

    #[test]
    fn test_get_hrir_left_ild() {
        let hrir = get_hrir_for_azimuth(90.0);
        // Left should be louder in left ear
        let left_energy: f32 = hrir.left.iter().map(|x| x * x).sum();
        let right_energy: f32 = hrir.right.iter().map(|x| x * x).sum();
        assert!(left_energy > right_energy, "Left source should be louder in left ear");
    }

    #[test]
    fn test_get_hrir_right_ild() {
        let hrir = get_hrir_for_azimuth(-90.0);
        // Right should be louder in right ear
        let left_energy: f32 = hrir.left.iter().map(|x| x * x).sum();
        let right_energy: f32 = hrir.right.iter().map(|x| x * x).sum();
        assert!(right_energy > left_energy, "Right source should be louder in right ear");
    }

    #[test]
    fn test_placeholder_hrir_not_silent() {
        // Verify placeholder HRIRs have some content
        let left_sum: f32 = HRIR_FRONT_LEFT.iter().map(|x| x.abs()).sum();
        assert!(left_sum > 0.1, "HRIR should not be silent");
    }
}
