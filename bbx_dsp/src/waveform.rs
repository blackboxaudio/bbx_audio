use bbx_core::random::XorShiftRng;

#[derive(Debug, Clone, Copy)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Pulse,
    Noise,
}

pub(crate) const DEFAULT_DUTY_CYCLE: f64 = 0.5;

const TWO_PI: f64 = 2.0 * std::f64::consts::PI;
const INV_TWO_PI: f64 = 1.0 / TWO_PI;

pub(crate) fn generate_waveform_sample(waveform: Waveform, phase: f64, duty_cycle: f64, rng: &mut XorShiftRng) -> f64 {
    match waveform {
        Waveform::Sine => phase.sin(),
        Waveform::Square => {
            if phase.sin() > 0.0 {
                1.0
            } else {
                -1.0
            }
        }
        Waveform::Sawtooth => {
            let normalized_phase = (phase % TWO_PI) * INV_TWO_PI;
            2.0 * normalized_phase - 1.0
        }
        Waveform::Triangle => {
            let normalized_phase = (phase % TWO_PI) * INV_TWO_PI;
            if normalized_phase < 0.5 {
                4.0 * normalized_phase - 1.0
            } else {
                3.0 - 4.0 * normalized_phase
            }
        }
        Waveform::Pulse => {
            let normalized_phase = (phase % TWO_PI) * INV_TWO_PI;
            if normalized_phase < duty_cycle { 1.0 } else { -1.0 }
        }
        Waveform::Noise => rng.next_noise_sample(),
    }
}
