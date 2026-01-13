//! Matrix-based binaural decoder using ILD (Interaural Level Difference) approximation.

use crate::graph::MAX_BLOCK_INPUTS;

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
    let energy_scale = 1.0 / 2.0_f64.sqrt();
    let num_channels = (order + 1) * (order + 1);

    for ear_coeffs in matrix.iter_mut() {
        for coeff in ear_coeffs.iter_mut().take(num_channels) {
            *coeff *= energy_scale;
        }
    }
}
