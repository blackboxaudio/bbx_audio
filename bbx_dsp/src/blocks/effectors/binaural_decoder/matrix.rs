//! Matrix-based binaural decoder using ILD (Interaural Level Difference) approximation.

use crate::{block::MAX_BLOCK_INPUTS, math};

/// Compute matrix decoder coefficients for the given ambisonic order.
///
/// Returns a 2×N matrix where N is the number of ambisonic channels.
/// Row 0 is left ear, row 1 is right ear.
pub fn compute_matrix(order: usize) -> [[f64; MAX_BLOCK_INPUTS]; 2] {
    let mut matrix = [[0.0; MAX_BLOCK_INPUTS]; 2];

    match order {
        1 => compute_foa_matrix(&mut matrix),
        2 => compute_soa_matrix(&mut matrix),
        3 => compute_toa_matrix(&mut matrix),
        _ => panic!("Ambisonic order must be 1, 2, or 3"),
    }

    normalize_matrix(&mut matrix, order);
    matrix
}

fn compute_foa_matrix(matrix: &mut [[f64; MAX_BLOCK_INPUTS]; 2]) {
    // ACN ordering: W(0), Y(1), Z(2), X(3)
    // Y is the lateral channel (positive = left, negative = right)
    // X is front-back (positive = front)
    // Z is up-down (minimal contribution for binaural)

    // Left ear
    matrix[0][0] = 0.5; // W (omnidirectional)
    matrix[0][1] = 0.5; // Y (positive for left)
    matrix[0][2] = 0.1; // Z (small vertical contribution)
    matrix[0][3] = 0.35; // X (front emphasis for externalization)

    // Right ear
    matrix[1][0] = 0.5; // W
    matrix[1][1] = -0.5; // Y (negative for right)
    matrix[1][2] = 0.1; // Z
    matrix[1][3] = 0.35; // X
}

fn compute_soa_matrix(matrix: &mut [[f64; MAX_BLOCK_INPUTS]; 2]) {
    // Start with scaled FOA components
    matrix[0][0] = 0.45; // W
    matrix[0][1] = 0.45; // Y
    matrix[0][2] = 0.09; // Z
    matrix[0][3] = 0.32; // X

    matrix[1][0] = 0.45;
    matrix[1][1] = -0.45;
    matrix[1][2] = 0.09;
    matrix[1][3] = 0.32;

    // Order 2 channels (ACN 4-8): V, T, R, S, U
    matrix[0][4] = 0.25; // V (left positive)
    matrix[0][5] = 0.08; // T
    matrix[0][6] = 0.05; // R
    matrix[0][7] = 0.08; // S
    matrix[0][8] = 0.15; // U

    matrix[1][4] = -0.25; // V (right negative)
    matrix[1][5] = 0.08;
    matrix[1][6] = 0.05;
    matrix[1][7] = 0.08;
    matrix[1][8] = 0.15;
}

fn compute_toa_matrix(matrix: &mut [[f64; MAX_BLOCK_INPUTS]; 2]) {
    // Left ear coefficients for all 16 channels
    let left: [f64; 16] = [
        // Order 0-1
        0.42, 0.42, 0.08, 0.30, // Order 2 (V, T, R, S, U)
        0.22, 0.07, 0.04, 0.07, 0.13, // Order 3 (Q, O, M, K, L, N, P)
        0.15, 0.10, 0.05, 0.03, 0.05, 0.10, 0.10,
    ];

    // Right ear: negate lateral components (Y-like channels)
    let right: [f64; 16] = [
        // Order 0-1 (Y negated)
        0.42, -0.42, 0.08, 0.30, // Order 2 (V negated)
        -0.22, 0.07, 0.04, 0.07, 0.13, // Order 3 (Q, O, N negated - lateral components)
        -0.15, -0.10, 0.05, 0.03, 0.05, -0.10, 0.10,
    ];

    matrix[0].copy_from_slice(&left);
    matrix[1].copy_from_slice(&right);
}

fn normalize_matrix(matrix: &mut [[f64; MAX_BLOCK_INPUTS]; 2], order: usize) {
    let energy_scale = 1.0 / math::sqrt(2.0_f64);
    let num_channels = (order + 1) * (order + 1);

    for ear_coeffs in matrix.iter_mut() {
        for coeff in ear_coeffs.iter_mut().take(num_channels) {
            *coeff *= energy_scale;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-10;
    const ENERGY_SCALE: f64 = 0.7071067811865476; // 1/sqrt(2)

    // ==================== FOA (order 1) tests ====================

    #[test]
    fn foa_returns_4_channels() {
        let matrix = compute_matrix(1);
        let num_channels = 4;

        for ch in 0..num_channels {
            assert!(
                matrix[0][ch].abs() > EPSILON || matrix[1][ch].abs() > EPSILON,
                "Channel {ch} should have non-zero coefficients"
            );
        }

        for ch in num_channels..MAX_BLOCK_INPUTS {
            assert!(
                matrix[0][ch].abs() < EPSILON && matrix[1][ch].abs() < EPSILON,
                "Channel {ch} should be zero for FOA"
            );
        }
    }

    #[test]
    fn foa_w_channel_is_symmetric() {
        let matrix = compute_matrix(1);

        assert!((matrix[0][0] - matrix[1][0]).abs() < EPSILON);
    }

    #[test]
    fn foa_y_channel_is_negated() {
        let matrix = compute_matrix(1);

        assert!((matrix[0][1] + matrix[1][1]).abs() < EPSILON);
        assert!(matrix[0][1] > 0.0);
        assert!(matrix[1][1] < 0.0);
    }

    #[test]
    fn foa_z_channel_is_symmetric() {
        let matrix = compute_matrix(1);

        assert!((matrix[0][2] - matrix[1][2]).abs() < EPSILON);
    }

    #[test]
    fn foa_x_channel_is_symmetric() {
        let matrix = compute_matrix(1);

        assert!((matrix[0][3] - matrix[1][3]).abs() < EPSILON);
    }

    #[test]
    fn foa_is_normalized() {
        let matrix = compute_matrix(1);

        let expected_w = 0.5 * ENERGY_SCALE;
        assert!((matrix[0][0] - expected_w).abs() < EPSILON);
    }

    // ==================== SOA (order 2) tests ====================

    #[test]
    fn soa_returns_9_channels() {
        let matrix = compute_matrix(2);
        let num_channels = 9;

        for ch in 0..num_channels {
            assert!(
                matrix[0][ch].abs() > EPSILON || matrix[1][ch].abs() > EPSILON,
                "Channel {ch} should have non-zero coefficients"
            );
        }

        for ch in num_channels..MAX_BLOCK_INPUTS {
            assert!(
                matrix[0][ch].abs() < EPSILON && matrix[1][ch].abs() < EPSILON,
                "Channel {ch} should be zero for SOA"
            );
        }
    }

    #[test]
    fn soa_y_channel_is_negated() {
        let matrix = compute_matrix(2);

        assert!((matrix[0][1] + matrix[1][1]).abs() < EPSILON);
    }

    #[test]
    fn soa_v_channel_is_negated() {
        let matrix = compute_matrix(2);

        assert!((matrix[0][4] + matrix[1][4]).abs() < EPSILON);
    }

    #[test]
    fn soa_is_normalized() {
        let matrix = compute_matrix(2);

        let expected_w = 0.45 * ENERGY_SCALE;
        assert!((matrix[0][0] - expected_w).abs() < EPSILON);
    }

    // ==================== TOA (order 3) tests ====================

    #[test]
    fn toa_returns_16_channels() {
        let matrix = compute_matrix(3);

        for ch in 0..16 {
            assert!(
                matrix[0][ch].abs() > EPSILON || matrix[1][ch].abs() > EPSILON,
                "Channel {ch} should have non-zero coefficients"
            );
        }
    }

    #[test]
    fn toa_y_channel_is_negated() {
        let matrix = compute_matrix(3);

        assert!((matrix[0][1] + matrix[1][1]).abs() < EPSILON);
    }

    #[test]
    fn toa_w_channel_is_symmetric() {
        let matrix = compute_matrix(3);

        assert!((matrix[0][0] - matrix[1][0]).abs() < EPSILON);
    }

    #[test]
    fn toa_is_normalized() {
        let matrix = compute_matrix(3);

        let expected_w = 0.42 * ENERGY_SCALE;
        assert!((matrix[0][0] - expected_w).abs() < EPSILON);
    }

    // ==================== invalid order tests ====================

    #[test]
    #[should_panic(expected = "Ambisonic order must be 1, 2, or 3")]
    fn order_0_panics() {
        compute_matrix(0);
    }

    #[test]
    #[should_panic(expected = "Ambisonic order must be 1, 2, or 3")]
    fn order_4_panics() {
        compute_matrix(4);
    }

    // ==================== matrix dimension tests ====================

    #[test]
    fn matrix_has_two_rows() {
        let matrix = compute_matrix(1);
        assert_eq!(matrix.len(), 2);
    }

    #[test]
    fn matrix_has_max_block_inputs_columns() {
        let matrix = compute_matrix(1);
        assert_eq!(matrix[0].len(), MAX_BLOCK_INPUTS);
        assert_eq!(matrix[1].len(), MAX_BLOCK_INPUTS);
    }

    // ==================== normalization tests ====================

    #[test]
    fn normalization_applies_energy_scale() {
        let mut matrix = [[0.0; MAX_BLOCK_INPUTS]; 2];
        matrix[0][0] = 1.0;
        matrix[1][0] = 1.0;

        normalize_matrix(&mut matrix, 1);

        assert!((matrix[0][0] - ENERGY_SCALE).abs() < EPSILON);
        assert!((matrix[1][0] - ENERGY_SCALE).abs() < EPSILON);
    }

    #[test]
    fn normalization_only_affects_relevant_channels() {
        let mut matrix = [[0.0; MAX_BLOCK_INPUTS]; 2];
        for i in 0..MAX_BLOCK_INPUTS {
            matrix[0][i] = 1.0;
            matrix[1][i] = 1.0;
        }

        normalize_matrix(&mut matrix, 1);

        for i in 0..4 {
            assert!(
                (matrix[0][i] - ENERGY_SCALE).abs() < EPSILON,
                "Channel {i} should be normalized"
            );
        }
        for i in 4..MAX_BLOCK_INPUTS {
            assert!(
                (matrix[0][i] - 1.0).abs() < EPSILON,
                "Channel {i} should not be normalized"
            );
        }
    }

    // ==================== left/right symmetry tests ====================

    #[test]
    fn all_orders_have_symmetric_w_channel() {
        for order in 1..=3 {
            let matrix = compute_matrix(order);
            assert!(
                (matrix[0][0] - matrix[1][0]).abs() < EPSILON,
                "Order {order} W channel should be symmetric"
            );
        }
    }

    #[test]
    fn all_orders_have_negated_y_channel() {
        for order in 1..=3 {
            let matrix = compute_matrix(order);
            assert!(
                (matrix[0][1] + matrix[1][1]).abs() < EPSILON,
                "Order {order} Y channel should be negated"
            );
        }
    }

    #[test]
    fn all_orders_have_symmetric_x_channel() {
        for order in 1..=3 {
            let matrix = compute_matrix(order);
            assert!(
                (matrix[0][3] - matrix[1][3]).abs() < EPSILON,
                "Order {order} X channel should be symmetric"
            );
        }
    }
}
