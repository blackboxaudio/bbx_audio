# Glossary

Audio DSP and bbx_audio terminology.

## A

**ADSR**: Attack-Decay-Sustain-Release. An envelope shape for amplitude or parameter control.

**Anti-aliasing**: Techniques to prevent aliasing artifacts when generating waveforms with sharp discontinuities. See PolyBLEP, PolyBLAMP.

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

**PolyBLAMP**: Polynomial Band-Limited rAMP. Anti-aliasing technique for waveforms with slope discontinuities (triangle waves). Applies polynomial corrections near transition points.

**PolyBLEP**: Polynomial Band-Limited stEP. Anti-aliasing technique for waveforms with step discontinuities (sawtooth, square, pulse waves). Applies polynomial corrections near transition points to reduce aliasing.

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

---

## Embedded Development Terms

### A

**ADC (Analog-to-Digital Converter)**: Hardware that converts analog voltages (knobs, CV inputs) to digital values.

### B

**BDMA (Basic DMA)**: DMA controller in the D3 power domain for low-power peripherals.

### C

**Cortex-M7**: ARM processor core used in the STM32H750. High-performance with FPU and DSP instructions.

### D

**DFU (Device Firmware Update)**: USB protocol for flashing firmware without a debug probe. Enter by holding BOOT during reset.

**DMA (Direct Memory Access)**: Hardware that transfers data between peripherals and memory without CPU intervention. Essential for audio streaming.

**DTCM (Data Tightly Coupled Memory)**: Fastest RAM region (128KB), zero wait states, but not DMA accessible.

### E

**EABI (Embedded ABI)**: Application Binary Interface for embedded ARM. Defines calling conventions and data layout.

### F

**FMC (Flexible Memory Controller)**: Peripheral that interfaces with external SDRAM.

**FPU (Floating Point Unit)**: Hardware for fast floating-point math. STM32H750 has single-precision FPU.

### G

**GPIO (General Purpose Input/Output)**: Pins that can be configured as digital inputs or outputs.

### H

**HAL (Hardware Abstraction Layer)**: Library providing ergonomic API over raw hardware registers.

**HSE (High-Speed External)**: External crystal oscillator (16 MHz on Daisy) used as clock source.

### I

**I2C (Inter-Integrated Circuit)**: Two-wire serial protocol for configuring codecs and peripherals.

**I2S (Inter-IC Sound)**: Serial protocol for digital audio between MCU and codec.

**ISR (Interrupt Service Routine)**: Function that runs in response to hardware events.

### L

**LLVM**: Compiler infrastructure used by Rust for code generation and optimization.

### M

**MCLK (Master Clock)**: Audio master clock, typically 256× the sample rate.

### N

**NVIC (Nested Vectored Interrupt Controller)**: ARM peripheral managing interrupt priorities and delivery.

### P

**PAC (Peripheral Access Crate)**: Auto-generated Rust crate providing raw register access.

**PLL (Phase-Locked Loop)**: Circuit that multiplies input frequency. Used to generate system and audio clocks.

### Q

**QSPI (Quad SPI)**: Four-line SPI interface for external flash memory (8MB on Daisy).

### R

**RTT (Real-Time Transfer)**: Debug channel for printf-style output via debug probe.

### S

**SAI (Serial Audio Interface)**: STM32H750 peripheral for I2S and other audio protocols.

**SDRAM**: External synchronous dynamic RAM (64MB on Daisy) for large buffers.

**SRAM**: Static RAM internal to the MCU. Multiple regions with different characteristics.

**SWD (Serial Wire Debug)**: Two-wire debug interface used by debug probes.

### T

**Thumb**: ARM instruction set used by Cortex-M processors. Mix of 16-bit and 32-bit instructions.

### V

**VCO (Voltage-Controlled Oscillator)**: Core of PLL that generates high-frequency clock.
