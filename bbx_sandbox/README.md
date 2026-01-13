# bbx_sandbox

Examples and testing playground for bbx_audio DSP development.

## Running Examples

```bash
cargo run --example <example_name> -p bbx_sandbox
```

## Examples

| Example | Description |
|---------|-------------|
| `01_sine_wave` | Basic sine wave oscillator |
| `02_pwm` | Pulse width modulation |
| `03_overdrive` | Overdrive distortion effect |
| `04_input_wav_file` | Read and play a WAV file |
| `05_output_wav_file` | Render audio to a WAV file |
| `05_midi_input` | MIDI input handling |
| `06_midi_synth` | Polyphonic MIDI synthesizer |
| `07_stereo_panner` | Stereo panning drone with LFO modulation |
| `08_ambisonic_panner` | FOA encoding with binaural decoding for headphones |
| `09_channel_routing` | ChannelRouterBlock with swap mode |
| `10_filter_modulation` | LFO-modulated low-pass filter (wah effect) |
| `11_channel_split_merge` | Parallel processing with splitter/merger |
| `12_effect_chain` | Multi-stage effect chain with modulation |
| `13_file_processing` | Offline WAV file processing pipeline |

## Quick Start

### Sine Wave

```bash
cargo run --example 01_sine_wave -p bbx_sandbox
```

### Filter Modulation

Classic wah/sweep effect using an LFO-modulated resonant filter:

```bash
cargo run --example 10_filter_modulation -p bbx_sandbox
```

### Spatial Audio (Headphones Recommended)

Ambisonic panning with binaural decoding for immersive 3D audio:

```bash
cargo run --example 08_ambisonic_panner -p bbx_sandbox
```
