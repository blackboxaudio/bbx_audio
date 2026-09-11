//! Lookup tables read by phase in cycles.
//!
//! Every table in this module follows one layout: `[f32; LENGTH]` holds
//! `LENGTH - 1` cells plus a guard entry at `table[LENGTH - 1]`. For a periodic
//! table the guard repeats `table[0]`; for an unwrapped curve it holds the final
//! height. Either way the last cell interpolates towards the guard, so the read
//! path never checks for wrap-around.
//!
//! The storage length is the const parameter (rather than the cell count) because
//! spelling `[f32; N + 1]` in a type needs the unstable `generic_const_exprs`
//! feature. `LENGTH - 1` is expected to be a power of two.

mod sine;

pub use sine::{SINE_2048, SineTable};

use crate::math;

/// Read `table` at `phase` (in cycles) with linear interpolation.
///
/// `phase` is wrapped into `[0, 1)` with `floor`, so any real value is valid and
/// `read_interpolated(table, p + 1.0)` reads the same cell as `p`.
///
/// Taking an array reference rather than a slice lets the compiler see the
/// length, so the two element loads compile without bounds checks.
#[inline]
pub fn read_interpolated<const LENGTH: usize>(table: &[f32; LENGTH], phase: f32) -> f32 {
    debug_assert!(LENGTH >= 2, "a table needs at least one cell and a guard entry");
    let cells = LENGTH - 1;

    let wrapped = phase - math::floor(phase);
    let position = wrapped * cells as f32;
    // `wrapped < 1.0` but `wrapped * cells` can still round up to exactly `cells` in f32;
    // clamping keeps `index + 1` inside the array and is what lets the bounds checks vanish.
    let index = (position as usize).min(cells - 1);
    let fraction = position - index as f32;

    let start = table[index];
    start + fraction * (table[index + 1] - start)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Four cells of a ramp `0, 1, 2, 3` with the guard holding the final height `4`.
    const RAMP: [f32; 5] = [0.0, 1.0, 2.0, 3.0, 4.0];

    #[test]
    fn test_reads_cell_boundaries_exactly() {
        for (index, expected) in RAMP[..4].iter().enumerate() {
            assert_eq!(read_interpolated(&RAMP, index as f32 / 4.0), *expected);
        }
    }

    #[test]
    fn test_interpolates_midpoints_exactly() {
        assert_eq!(read_interpolated(&RAMP, 0.125), 0.5);
        assert_eq!(read_interpolated(&RAMP, 0.375), 1.5);
        assert_eq!(read_interpolated(&RAMP, 0.875), 3.5);
    }

    #[test]
    fn test_last_cell_interpolates_towards_guard() {
        let just_below_one = 1.0 - f32::EPSILON;
        let value = read_interpolated(&RAMP, just_below_one);
        assert!(value > 3.9 && value <= 4.0, "got {value}");
    }

    #[test]
    fn test_wraps_phase_with_floor() {
        for phase in [0.125f32, 0.375, 0.875] {
            let base = read_interpolated(&RAMP, phase);
            assert_eq!(read_interpolated(&RAMP, phase + 1.0), base);
            assert_eq!(read_interpolated(&RAMP, phase + 3.0), base);
            assert_eq!(read_interpolated(&RAMP, phase - 1.0), base);
            assert_eq!(read_interpolated(&RAMP, phase - 2.0), base);
        }
    }

    #[test]
    fn test_single_cell_table() {
        let table = [1.0f32, 3.0];
        assert_eq!(read_interpolated(&table, 0.0), 1.0);
        assert_eq!(read_interpolated(&table, 0.5), 2.0);
    }
}
