# Glossary

Audio DSP and bbx_audio terminology.

## A

**ADSR**: Attack-Decay-Sustain-Release. An envelope shape for amplitude or parameter control.

**Audio Thread**: The high-priority thread that processes audio. Must be real-time safe.

## B

**Block**: A DSP processing unit in bbx_audio. Implements the `Block` trait.

**Buffer**: A fixed-size array of audio samples, typically 256-2048 samples.

**Buffer Size**: Number of samples processed per audio callback.

## C

**Control Rate**: Updating values once per buffer, not per sample. Used for modulation.

**Cycle**: An illegal loop in a DSP graph where a block's output feeds back to its input.

## D

**DAG**: Directed Acyclic Graph. The structure of a DSP graph with no cycles.

**Denormal**: Very small floating-point numbers that cause CPU slowdowns.

**DSP**: Digital Signal Processing. Mathematical operations on audio samples.

## E

**Effector**: A block that processes audio (gain, filter, distortion).

**Envelope**: A time-varying control signal, typically ADSR.

## F

**FFI**: Foreign Function Interface. How Rust code is called from C/C++.

## G

**Generator**: A block that creates audio from nothing (oscillator, noise).

**Graph**: A connected set of DSP blocks with defined signal flow.

## L

**Latency**: Delay between input and output, measured in samples or milliseconds.

**LFO**: Low-Frequency Oscillator. A slow oscillator for modulation.

## M

**Modulation**: Varying a parameter over time using a control signal.

**Modulator**: A block that generates control signals (LFO, envelope).

## O

**Oscillator**: A block that generates periodic waveforms.

## P

**Port**: An input or output connection point on a block.

## R

**Real-Time Safe**: Code that completes in bounded time without blocking.

## S

**Sample**: A single audio value at a point in time.

**Sample Rate**: Samples per second, typically 44100 or 48000 Hz.

**SIMD**: Single Instruction Multiple Data. Processing multiple samples at once.

**SPSC**: Single-Producer Single-Consumer. A lock-free queue pattern.

## T

**Topological Sort**: Algorithm that orders blocks so dependencies run first.

## W

**Waveform**: The shape of a periodic signal (sine, square, saw, triangle).
