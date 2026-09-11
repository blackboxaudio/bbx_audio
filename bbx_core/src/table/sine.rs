//! A sine table evaluated at compile time.

use super::read_interpolated;

/// A sine wave sampled over one cycle, built at compile time.
///
/// `LENGTH - 1` cells plus a guard entry that repeats cell zero (see the
/// [`table`](crate::table) module for the layout). The cell count must be a power
/// of two, so the everyday instance is [`SINE_2048`] (`SineTable<2049>`, 8 KB).
///
/// # Accuracy
///
/// Linear interpolation with step `h = 2π / N` has error bounded by
/// `h² / 8 · max|sin''| = (2π / N)² / 8`:
///
/// | cells | bound    | level   |
/// |-------|----------|---------|
/// | 1024  | 4.7e-6   | −106 dB |
/// | 2048  | 1.2e-6   | −118 dB |
/// | 4096  | 3.0e-7   | −130 dB |
///
/// The entries themselves are rounded from `f64` and sit within `1e-6` of the
/// true sine. Tables beyond 4096 cells stop fitting in L1 cache and read slower,
/// not better.
///
/// # Building
///
/// `f64::sin` is not a `const fn`, so entries come from a Taylor series on a
/// quarter cycle (terms through `x¹¹ / 11!`, whose first dropped term at `π/2`
/// is about `5.7e-8`, below `f32` epsilon) and the remaining three quarters
/// from symmetry.
///
/// ```
/// use bbx_core::{SINE_2048, SineTable};
///
/// static QUIET: SineTable<1025> = SineTable::new();
///
/// assert!((SINE_2048.read(0.25) - 1.0).abs() < 1e-6);
/// assert!((QUIET.read_cosine(0.0) - 1.0).abs() < 1e-6);
/// ```
#[derive(Debug, Clone)]
pub struct SineTable<const LENGTH: usize> {
    entries: [f32; LENGTH],
}

/// One cycle of sine over 2048 cells.
///
/// A `static` rather than a `const` so every reader shares this one copy; a
/// `const` would be re-materialised (an 8 KB copy) wherever it is used by value.
pub static SINE_2048: SineTable<2049> = SineTable::new();

// No `Default`: it would rebuild the table at runtime (thousands of f64 Taylor
// iterations, soft-float on Cortex-M7), which must be a deliberate `new()` call.
#[allow(clippy::new_without_default)]
impl<const LENGTH: usize> SineTable<LENGTH> {
    /// Number of cells, one less than the storage length.
    pub const CELLS: usize = LENGTH - 1;

    /// Evaluate the table. Usable in `const` and `static` initialisers.
    ///
    /// # Panics
    ///
    /// At compile time (or at runtime if called there) when `LENGTH - 1` is not
    /// a power of two.
    pub const fn new() -> Self {
        assert!(LENGTH >= 2, "SineTable needs at least one cell and a guard entry");
        assert!(
            (LENGTH - 1).is_power_of_two(),
            "SineTable cell count (LENGTH - 1) must be a power of two"
        );

        let cells = LENGTH - 1;
        let mut entries = [0.0f32; LENGTH];
        let mut index = 0;
        while index < cells {
            entries[index] = sine_of_cycles(index as f64 / cells as f64) as f32;
            index += 1;
        }
        entries[cells] = entries[0];

        Self { entries }
    }

    /// `sin(2π · phase)` with `phase` in cycles, wrapped and interpolated.
    #[inline]
    pub fn read(&self, phase: f32) -> f32 {
        read_interpolated(&self.entries, phase)
    }

    /// `cos(2π · phase)`, read from the same table a quarter cycle ahead.
    #[inline]
    pub fn read_cosine(&self, phase: f32) -> f32 {
        self.read(phase + 0.25)
    }

    /// The raw entries, guard included.
    #[inline]
    pub const fn entries(&self) -> &[f32; LENGTH] {
        &self.entries
    }
}

/// `sin(2π · cycles)` for `cycles` in `[0, 1)`, reduced to a quarter cycle by symmetry.
const fn sine_of_cycles(cycles: f64) -> f64 {
    let (sign, half) = if cycles >= 0.5 {
        (-1.0, cycles - 0.5)
    } else {
        (1.0, cycles)
    };
    let quarter = if half > 0.25 { 0.5 - half } else { half };
    sign * taylor_sin_quarter(core::f64::consts::TAU * quarter)
}

/// Taylor series for `sin(x)` on `[0, π/2]`.
const fn taylor_sin_quarter(x: f64) -> f64 {
    let x_squared = x * x;
    let mut term = x;
    let mut sum = x;
    let mut k = 1.0;
    while k < 12.0 {
        term *= -x_squared / ((k + 1.0) * (k + 2.0));
        sum += term;
        k += 2.0;
    }
    sum
}

#[cfg(test)]
mod tests {
    use core::f64::consts::TAU;

    use super::*;
    use crate::random::XorShiftRng;

    const ENTRY_TOLERANCE: f64 = 1e-6;
    const READ_TOLERANCE: f64 = 2e-6;

    fn true_sine(cycles: f64) -> f64 {
        libm::sin(TAU * cycles)
    }

    fn check_entries<const LENGTH: usize>() {
        let table = SineTable::<LENGTH>::new();
        let cells = LENGTH - 1;
        for (index, entry) in table.entries().iter().enumerate() {
            let expected = true_sine(index as f64 / cells as f64);
            let error = (*entry as f64 - expected).abs();
            assert!(
                error < ENTRY_TOLERANCE,
                "{LENGTH}: entry {index} = {entry}, expected {expected}, error {error}"
            );
        }
        assert_eq!(table.entries()[cells], table.entries()[0]);
    }

    #[test]
    fn test_entries_match_libm_sine() {
        check_entries::<1025>();
        check_entries::<2049>();
        check_entries::<4097>();
    }

    #[test]
    fn test_builds_in_const_context() {
        const TABLE: SineTable<2049> = SineTable::new();
        assert_eq!(TABLE.entries().len(), 2049);
        assert_eq!(SineTable::<2049>::CELLS, 2048);
        assert!((SINE_2048.read(0.25) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_read_at_random_phases() {
        let mut rng = XorShiftRng::new(0x5EED);
        for _ in 0..10_000 {
            let phase = rng.next_noise_sample() * 4.0;
            let actual = SINE_2048.read(phase as f32) as f64;
            let expected = true_sine(phase);
            let error = (actual - expected).abs();
            assert!(
                error < READ_TOLERANCE,
                "phase {phase}: {actual} vs {expected}, error {error}"
            );
        }
    }

    #[test]
    fn test_read_is_periodic_exactly_on_dyadic_phases() {
        // `k / 1024 ± 1.0` and the floor subtraction are exact in f32, so the wrapped phase is
        // bit-identical; for arbitrary phases `(p + 1.0) - 1.0 != p` and only a tolerance holds.
        for k in 0..1024 {
            let phase = k as f32 / 1024.0;
            let base = SINE_2048.read(phase);
            assert_eq!(SINE_2048.read(phase + 1.0), base, "phase {phase} + 1");
            assert_eq!(SINE_2048.read(phase - 1.0), base, "phase {phase} - 1");
        }
    }

    #[test]
    fn test_landmark_phases() {
        assert_eq!(SINE_2048.read(0.0), 0.0);
        assert!((SINE_2048.read(0.25) - 1.0).abs() < 1e-6);
        assert!(SINE_2048.read(0.5).abs() < 1e-6);
        assert!((SINE_2048.read(0.75) + 1.0).abs() < 1e-6);
        assert!((SINE_2048.read_cosine(0.0) - 1.0).abs() < 1e-6);
        assert!(SINE_2048.read_cosine(0.25).abs() < 1e-6);
    }

    #[test]
    fn test_last_cell_before_wrap() {
        let value = SINE_2048.read(1.0 - f32::EPSILON);
        assert!(value.is_finite());
        assert!(value.abs() < 1e-5, "got {value}");
    }

    #[test]
    fn test_read_stays_in_unit_range() {
        for k in 0..100_000 {
            let value = SINE_2048.read(k as f32 * 1e-5);
            assert!((-1.0..=1.0).contains(&value), "phase {k}e-5 read {value}");
        }
    }

    #[test]
    #[should_panic(expected = "power of two")]
    fn test_rejects_non_power_of_two_cell_count() {
        let _ = SineTable::<1000>::new();
    }
}
