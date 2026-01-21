//! ADC abstractions for potentiometers and CV inputs.
//!
//! This module provides convenient wrappers for reading analog inputs
//! with optional smoothing and scaling.

// ============================================================================
// Knob (Potentiometer) Abstraction
// ============================================================================

/// Potentiometer/knob input with exponential smoothing.
///
/// Provides filtered analog readings suitable for controlling parameters
/// without jitter or zipper noise.
///
/// # Example
///
/// ```ignore
/// use bbx_daisy::peripherals::adc::Knob;
///
/// let mut knob = Knob::new(0.1); // 10% smoothing factor
///
/// // In control loop:
/// let raw_value = adc.read(&mut knob_pin);
/// let smoothed = knob.process(raw_value);
/// filter.set_cutoff(smoothed * 20000.0); // 0-20kHz range
/// ```
pub struct Knob {
    /// Current smoothed value (0.0 to 1.0)
    value: f32,
    /// Smoothing coefficient (0.0 = no smoothing, 1.0 = infinite smoothing)
    alpha: f32,
    /// Deadzone at min/max positions (prevents noise at extremes)
    deadzone: f32,
}

impl Knob {
    /// Create a new knob with the specified smoothing factor.
    ///
    /// # Arguments
    ///
    /// * `alpha` - Smoothing coefficient (0.0 to 1.0)
    ///   - Lower values = faster response, more jitter
    ///   - Higher values = slower response, smoother output
    ///   - Typical values: 0.05 (fast) to 0.2 (smooth)
    pub fn new(alpha: f32) -> Self {
        Self {
            value: 0.0,
            alpha: alpha.clamp(0.0, 0.99),
            deadzone: 0.005, // 0.5% deadzone at extremes
        }
    }

    /// Create with default smoothing (0.1).
    pub fn default_smoothing() -> Self {
        Self::new(0.1)
    }

    /// Set the deadzone for min/max positions.
    pub fn with_deadzone(mut self, deadzone: f32) -> Self {
        self.deadzone = deadzone.clamp(0.0, 0.1);
        self
    }

    /// Set the smoothing coefficient.
    pub fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha.clamp(0.0, 0.99);
    }

    /// Process a raw ADC reading (16-bit) and return smoothed 0.0-1.0 value.
    ///
    /// # Arguments
    ///
    /// * `raw` - Raw ADC value (0 to 65535 for 16-bit ADC)
    #[inline]
    pub fn process_u16(&mut self, raw: u16) -> f32 {
        let normalized = raw as f32 / 65535.0;
        self.process_normalized(normalized)
    }

    /// Process a raw ADC reading (12-bit) and return smoothed 0.0-1.0 value.
    ///
    /// # Arguments
    ///
    /// * `raw` - Raw ADC value (0 to 4095 for 12-bit ADC)
    #[inline]
    pub fn process_u12(&mut self, raw: u16) -> f32 {
        let normalized = (raw & 0xFFF) as f32 / 4095.0;
        self.process_normalized(normalized)
    }

    /// Process a normalized (0.0-1.0) input value.
    #[inline]
    pub fn process_normalized(&mut self, input: f32) -> f32 {
        // Apply deadzone at extremes
        let clamped = if input < self.deadzone {
            0.0
        } else if input > 1.0 - self.deadzone {
            1.0
        } else {
            // Scale the middle range to 0.0-1.0
            (input - self.deadzone) / (1.0 - 2.0 * self.deadzone)
        };

        // Exponential moving average filter
        self.value = self.alpha * self.value + (1.0 - self.alpha) * clamped;
        self.value
    }

    /// Get the current smoothed value without processing new input.
    #[inline]
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Reset to a specific value (useful for initialization).
    pub fn reset(&mut self, value: f32) {
        self.value = value.clamp(0.0, 1.0);
    }

    /// Map the knob value to a custom range.
    #[inline]
    pub fn map_range(&self, min: f32, max: f32) -> f32 {
        min + self.value * (max - min)
    }

    /// Map the knob value with exponential scaling (useful for frequency controls).
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum output value
    /// * `max` - Maximum output value
    /// * `curve` - Exponential curve (1.0 = linear, 2.0 = quadratic, etc.)
    #[inline]
    pub fn map_exponential(&self, min: f32, max: f32, curve: f32) -> f32 {
        let curved = libm::powf(self.value, curve);
        min + curved * (max - min)
    }
}

impl Default for Knob {
    fn default() -> Self {
        Self::default_smoothing()
    }
}

// ============================================================================
// CV Input (for Eurorack)
// ============================================================================

/// CV (Control Voltage) input for Eurorack modules.
///
/// Handles bipolar (-5V to +5V) or unipolar (0V to 5V/10V) CV signals
/// with optional smoothing and range configuration.
pub struct CvInput {
    /// Current smoothed value
    value: f32,
    /// Smoothing coefficient
    alpha: f32,
    /// Input range configuration
    range: CvRange,
}

/// CV input voltage range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CvRange {
    /// 0V to 3.3V (Daisy Seed direct)
    Unipolar3V3,
    /// 0V to 5V (with voltage divider)
    Unipolar5V,
    /// -5V to +5V (Eurorack standard, with offset circuit)
    Bipolar5V,
    /// 0V to 10V (with voltage divider)
    Unipolar10V,
}

impl CvInput {
    /// Create a new CV input with specified range and smoothing.
    pub fn new(range: CvRange, alpha: f32) -> Self {
        Self {
            value: 0.0,
            alpha: alpha.clamp(0.0, 0.99),
            range,
        }
    }

    /// Create a bipolar (-5V to +5V) CV input.
    pub fn bipolar(alpha: f32) -> Self {
        Self::new(CvRange::Bipolar5V, alpha)
    }

    /// Create a unipolar (0V to 5V) CV input.
    pub fn unipolar(alpha: f32) -> Self {
        Self::new(CvRange::Unipolar5V, alpha)
    }

    /// Process a 12-bit ADC reading and return the voltage.
    ///
    /// Returns the voltage in the configured range.
    #[inline]
    pub fn process_u12(&mut self, raw: u16) -> f32 {
        let normalized = (raw & 0xFFF) as f32 / 4095.0;

        let voltage = match self.range {
            CvRange::Unipolar3V3 => normalized * 3.3,
            CvRange::Unipolar5V => normalized * 5.0,
            CvRange::Bipolar5V => (normalized - 0.5) * 10.0, // -5V to +5V
            CvRange::Unipolar10V => normalized * 10.0,
        };

        // Apply smoothing
        self.value = self.alpha * self.value + (1.0 - self.alpha) * voltage;
        self.value
    }

    /// Get the current smoothed voltage value.
    #[inline]
    pub fn voltage(&self) -> f32 {
        self.value
    }

    /// Get the value normalized to 0.0-1.0 range.
    #[inline]
    pub fn normalized(&self) -> f32 {
        match self.range {
            CvRange::Unipolar3V3 => self.value / 3.3,
            CvRange::Unipolar5V => self.value / 5.0,
            CvRange::Bipolar5V => (self.value + 5.0) / 10.0,
            CvRange::Unipolar10V => self.value / 10.0,
        }
    }

    /// Get the value as a 1V/octave pitch offset in semitones.
    ///
    /// Assumes 1V = 1 octave = 12 semitones.
    #[inline]
    pub fn as_semitones(&self) -> f32 {
        self.value * 12.0
    }

    /// Reset to zero voltage.
    pub fn reset(&mut self) {
        self.value = 0.0;
    }
}

impl Default for CvInput {
    fn default() -> Self {
        Self::bipolar(0.1)
    }
}

// ============================================================================
// Multi-Channel ADC Reading
// ============================================================================

/// Collection of multiple ADC inputs for batch processing.
///
/// Useful when reading multiple knobs or CV inputs together.
pub struct AdcInputs<const N: usize> {
    knobs: [Knob; N],
}

impl<const N: usize> AdcInputs<N> {
    /// Create a new collection with uniform smoothing.
    pub fn new(alpha: f32) -> Self {
        Self {
            knobs: core::array::from_fn(|_| Knob::new(alpha)),
        }
    }

    /// Process all inputs from a slice of raw 12-bit values.
    pub fn process_all(&mut self, raw_values: &[u16; N]) -> [f32; N] {
        let mut output = [0.0f32; N];
        for i in 0..N {
            output[i] = self.knobs[i].process_u12(raw_values[i]);
        }
        output
    }

    /// Get a reference to a specific knob.
    pub fn knob(&self, index: usize) -> &Knob {
        &self.knobs[index]
    }

    /// Get a mutable reference to a specific knob.
    pub fn knob_mut(&mut self, index: usize) -> &mut Knob {
        &mut self.knobs[index]
    }
}

impl<const N: usize> Default for AdcInputs<N> {
    fn default() -> Self {
        Self::new(0.1)
    }
}
