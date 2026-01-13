#!/usr/bin/env python3
"""Extract MIT KEMAR HRIR data from SOFA file and generate Rust code."""

import h5py
import numpy as np

SOFA_FILE = "mit_kemar.sofa"
OUTPUT_FILE = "hrir_data_kemar.rs"
HRIR_LENGTH = 256

TARGET_AZIMUTHS = [
    ("FRONT", 0.0),
    ("FRONT_LEFT_45", 45.0),
    ("LEFT", 90.0),
    ("REAR_LEFT", 135.0),
    ("REAR", 180.0),
    ("FRONT_RIGHT_45", -45.0),
    ("RIGHT", -90.0),
    ("REAR_RIGHT", -135.0),
]

def normalize_azimuth(az):
    """Normalize azimuth to 0-360 range."""
    return az % 360.0

def find_closest_position(positions, target_az, target_el=0.0):
    """Find the measurement closest to the target azimuth and elevation."""
    best_idx = 0
    best_dist = float('inf')

    for i, pos in enumerate(positions):
        az = normalize_azimuth(pos[0])
        el = pos[1]

        norm_target = normalize_azimuth(target_az)

        az_diff = abs(az - norm_target)
        az_diff = min(az_diff, 360.0 - az_diff)

        el_diff = abs(el - target_el)

        # Weight elevation difference more heavily
        dist = az_diff**2 + (el_diff * 2)**2

        if dist < best_dist:
            best_dist = dist
            best_idx = i

    return best_idx

def format_array(name, ear, samples):
    """Format a single HRIR array as Rust code."""
    lines = [f"static HRIR_{name}_{ear}: [f32; HRIR_LENGTH] = ["]

    for i in range(0, len(samples), 8):
        chunk = samples[i:i+8]
        values = ", ".join(f"{v:>13.10}" for v in chunk)
        lines.append(f"    {values},")

    lines.append("];")
    return "\n".join(lines) + "\n\n"

def main():
    print(f"Reading SOFA file: {SOFA_FILE}")

    with h5py.File(SOFA_FILE, 'r') as f:
        # Print dataset info
        print("Datasets in file:")
        for key in f.keys():
            print(f"  {key}: {f[key].shape if hasattr(f[key], 'shape') else 'N/A'}")

        source_positions = f['SourcePosition'][:]
        ir_data = f['Data.IR'][:]

        print(f"\nSource positions shape: {source_positions.shape}")
        print(f"IR data shape: {ir_data.shape}")

        num_samples = ir_data.shape[2]
        print(f"Samples per HRIR: {num_samples}")

    # Generate Rust code
    output = []

    # Header
    output.append("//! Head-Related Impulse Response (HRIR) data for binaural decoding.")
    output.append("//!")
    output.append("//! # Attribution")
    output.append("//! HRIR data from MIT Media Lab KEMAR HRTF Database.")
    output.append("//! Bill Gardner and Keith Martin, MIT Media Lab, 1994.")
    output.append("//! <https://sound.media.mit.edu/resources/KEMAR.html>")
    output.append("//!")
    output.append("//! SOFA format provided by sofacoustics.org.")
    output.append("")
    output.append(f"/// Length of the HRIRs in samples.")
    output.append(f"pub const HRIR_LENGTH: usize = {HRIR_LENGTH};")
    output.append("")
    output.append("/// HRIR pair (left and right ear).")
    output.append("pub struct HrirPair {")
    output.append("    pub left: &'static [f32; HRIR_LENGTH],")
    output.append("    pub right: &'static [f32; HRIR_LENGTH],")
    output.append("}")
    output.append("")

    # Extract HRIRs for each target position
    for name, target_az in TARGET_AZIMUTHS:
        idx = find_closest_position(source_positions, target_az, 0.0)
        found_az = source_positions[idx][0]
        found_el = source_positions[idx][1]
        print(f"  {name} (target {target_az:.0f}°): found idx={idx} at az={found_az:.1f}°, el={found_el:.1f}°")

        # Extract left and right ear HRIRs
        left_samples = ir_data[idx, 0, :HRIR_LENGTH].astype(np.float32)
        right_samples = ir_data[idx, 1, :HRIR_LENGTH].astype(np.float32)

        # Pad if necessary
        if len(left_samples) < HRIR_LENGTH:
            left_samples = np.pad(left_samples, (0, HRIR_LENGTH - len(left_samples)))
            right_samples = np.pad(right_samples, (0, HRIR_LENGTH - len(right_samples)))

        output.append(format_array(name, "LEFT", left_samples))
        output.append(format_array(name, "RIGHT", right_samples))

    # Generate get_hrir_for_azimuth function
    output.append("""/// Get HRIR pair for a given azimuth (elevation assumed 0).
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
            left: &HRIR_LEFT_LEFT,
            right: &HRIR_LEFT_RIGHT,
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
            left: &HRIR_RIGHT_LEFT,
            right: &HRIR_RIGHT_RIGHT,
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
""")

    # Generate tests
    output.append("""#[cfg(test)]
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
        assert!(ratio > 0.5 && ratio < 2.0, "Front should be roughly balanced, ratio={}", ratio);
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
    fn test_kemar_hrir_not_silent() {
        // Verify KEMAR HRIRs have actual content
        let hrir = get_hrir_for_azimuth(0.0);
        let left_energy: f32 = hrir.left.iter().map(|x| x * x).sum();
        assert!(left_energy > 0.0001, "KEMAR HRIR should have content");
    }
}
""")

    # Write output
    with open(OUTPUT_FILE, 'w') as f:
        f.write("\n".join(output))

    print(f"\nGenerated: {OUTPUT_FILE}")
    print(f"Copy this file to bbx_dsp/src/blocks/effectors/binaural_decoder/hrir_data.rs")

if __name__ == "__main__":
    main()
