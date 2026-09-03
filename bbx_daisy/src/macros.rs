//! Entry point macros for Daisy applications.
//!
//! These macros eliminate boilerplate by handling:
//! - Entry point setup (`#[cortex_m_rt::entry]`)
//! - Panic handler (`panic_halt`)
//! - Audio callback registration
//! - Control input reading (ADC)
//! - Main loop with `wfi()`

/// Entry point macro for audio processing applications.
///
/// This macro creates a complete entry point for Daisy audio applications.
/// It handles all unsafe static state management internally, including:
/// - Clock configuration with PLL3 for SAI audio
/// - SAI1 + DMA initialization
/// - Audio callback registration
/// - Main loop with `wfi()`
///
/// No hardware controls are read: the `controls` passed to `process` stay at
/// their [`Controls::new`](crate::Controls::new) defaults. Use
/// [`bbx_daisy_audio_with_controls!`](crate::bbx_daisy_audio_with_controls)
/// when the board's knobs, CVs, gates, or buttons should be live.
///
/// # Usage
///
/// Pass the processor type and an expression that creates it:
///
/// ```ignore
/// #![no_std]
/// #![no_main]
///
/// use bbx_daisy::{bbx_daisy_audio, prelude::*};
///
/// struct FixedSine {
///     phase: f32,
/// }
///
/// impl AudioProcessor for FixedSine {
///     fn process(
///         &mut self,
///         _input: &FrameBuffer<BLOCK_SIZE>,
///         output: &mut FrameBuffer<BLOCK_SIZE>,
///         _controls: &Controls,
///     ) {
///         let phase_inc = 440.0 / DEFAULT_SAMPLE_RATE;
///
///         for i in 0..BLOCK_SIZE {
///             let sample = sinf(self.phase * 2.0 * PI) * 0.5;
///             output.set_frame(i, sample, sample);
///             self.phase += phase_inc;
///             if self.phase >= 1.0 {
///                 self.phase -= 1.0;
///             }
///         }
///     }
/// }
///
/// bbx_daisy_audio!(FixedSine, FixedSine { phase: 0.0 });
/// ```
#[macro_export]
macro_rules! bbx_daisy_audio {
    ($processor_type:ty, $processor_init:expr) => {
        use $crate::__internal::panic_halt as _;

        static mut __BBX_PROCESSOR: core::mem::MaybeUninit<$processor_type> = core::mem::MaybeUninit::uninit();
        static __BBX_CONTROLS: $crate::controls::AtomicControls = $crate::controls::AtomicControls::new();

        fn __bbx_audio_callback(
            input: &$crate::FrameBuffer<{ $crate::audio::BLOCK_SIZE }>,
            output: &mut $crate::FrameBuffer<{ $crate::audio::BLOCK_SIZE }>,
        ) {
            // Race-free snapshot of the control state (main loop is the writer).
            let controls = __BBX_CONTROLS.load();

            unsafe {
                // Call user's audio processor with controls
                let processor = __BBX_PROCESSOR.assume_init_mut();
                $crate::AudioProcessor::process(processor, input, output, &controls);
            }
        }

        #[$crate::__internal::entry]
        fn main() -> ! {
            unsafe {
                __BBX_PROCESSOR.write($processor_init);
            }

            // Initialize the board's audio hardware (clocks, codec, SAI pins).
            let audio = $crate::board::init_audio().expect("Failed to initialize audio hardware");

            // Let the processor precompute sample-rate-dependent state before streaming.
            unsafe {
                let processor = __BBX_PROCESSOR.assume_init_mut();
                $crate::AudioProcessor::prepare(processor, $crate::audio::DEFAULT_SAMPLE_RATE);
            }

            // Register the audio callback and start SAI + DMA streaming.
            $crate::audio::set_callback(__bbx_audio_callback).expect("audio already running");
            $crate::audio::init_and_start(
                audio.sample_rate,
                audio.sai1,
                audio.dma1,
                audio.dma1_rec,
                audio.sai1_pins,
                audio.sai1_rec,
                &audio.clocks,
            )
            .expect("Failed to start audio streaming");

            loop {
                $crate::__internal::wfi();
            }
        }
    };
}

/// Entry point macro for audio processing with ADC control inputs.
///
/// Pod variant: initializes ADC1 for the two knobs (knob 1 = PC4, knob 2 = PC0)
/// and populates `controls.knobs[0..2]` (smoothed, 0.0-1.0) in the main loop
/// for use in `AudioProcessor::process()`.
///
/// # Example
///
/// ```ignore
/// #![no_std]
/// #![no_main]
///
/// use bbx_daisy::{bbx_daisy_audio_with_controls, prelude::*};
///
/// struct TunableSine { phase: f32 }
///
/// impl AudioProcessor for TunableSine {
///     fn process(
///         &mut self,
///         _input: &FrameBuffer<BLOCK_SIZE>,
///         output: &mut FrameBuffer<BLOCK_SIZE>,
///         controls: &Controls,
///     ) {
///         let freq = 110.0 + controls.knobs[0] * 770.0;
///         // ...
///     }
/// }
///
/// bbx_daisy_audio_with_controls!(TunableSine, TunableSine { phase: 0.0 });
/// ```
#[cfg(feature = "pod")]
#[macro_export]
macro_rules! bbx_daisy_audio_with_controls {
    ($processor_type:ty, $processor_init:expr) => {
        use $crate::__internal::panic_halt as _;

        static mut __BBX_PROCESSOR: core::mem::MaybeUninit<$processor_type> = core::mem::MaybeUninit::uninit();
        static __BBX_CONTROLS: $crate::controls::AtomicControls = $crate::controls::AtomicControls::new();

        fn __bbx_audio_callback(
            input: &$crate::FrameBuffer<{ $crate::audio::BLOCK_SIZE }>,
            output: &mut $crate::FrameBuffer<{ $crate::audio::BLOCK_SIZE }>,
        ) {
            // Race-free snapshot of the control state (main loop is the writer).
            let controls = __BBX_CONTROLS.load();

            unsafe {
                // Call user's audio processor with controls
                let processor = __BBX_PROCESSOR.assume_init_mut();
                $crate::AudioProcessor::process(processor, input, output, &controls);
            }
        }

        #[$crate::__internal::entry]
        fn main() -> ! {
            unsafe {
                __BBX_PROCESSOR.write($processor_init);
            }

            // Initialize board with ADC for knob reading (codec auto-detected at runtime)
            let board = $crate::board::init_audio_with_adc().expect("Failed to initialize audio board with ADC");

            // Let the processor precompute sample-rate-dependent state before streaming.
            unsafe {
                let processor = __BBX_PROCESSOR.assume_init_mut();
                $crate::AudioProcessor::prepare(processor, $crate::audio::DEFAULT_SAMPLE_RATE);
            }

            // Set the audio callback
            $crate::audio::set_callback(__bbx_audio_callback).expect("audio already running");

            // Destructure board to extract audio peripherals and ADC components
            let $crate::board::AudioBoardWithAdc {
                audio,
                mut adc1,
                mut knob1_pin,
                mut knob2_pin,
            } = board;

            // Start audio processing (consumes audio peripherals)
            $crate::audio::init_and_start(
                audio.sample_rate,
                audio.sai1,
                audio.dma1,
                audio.dma1_rec,
                audio.sai1_pins,
                audio.sai1_rec,
                &audio.clocks,
            )
            .expect("Failed to start audio streaming");

            // Knob smoothing state lives in the main loop — only the atomic
            // controls store is shared with the ISR.
            let mut knob1 = $crate::peripherals::Knob::default_smoothing_const();
            let mut knob2 = $crate::peripherals::Knob::default_smoothing_const();

            // Main loop: read ADC and update controls
            loop {
                // Read knobs and update controls
                // ADC returns u32, shift down to 12-bit range for processing
                let raw1 = (adc1.read(&mut knob1_pin).unwrap_or(0_u32) >> 4) as u16;
                let raw2 = (adc1.read(&mut knob2_pin).unwrap_or(0_u32) >> 4) as u16;

                // Smooth and publish to the ISR-visible atomic store.
                __BBX_CONTROLS.set_knob(0, knob1.process_u12(raw1));
                __BBX_CONTROLS.set_knob(1, knob2.process_u12(raw2));

                $crate::__internal::wfi();
            }
        }
    };
}

/// Patch.Init (`patch_sm`) variant of [`bbx_daisy_audio_with_controls`].
///
/// Brings up the full Patch.Init() control surface and populates every
/// [`Controls`](crate::Controls) field for
/// [`AudioProcessor::process`](crate::AudioProcessor::process):
///
/// | `Controls` field | Hardware | Conditioning |
/// |---|---|---|
/// | `knobs[0..4]` | panel knobs 1-4 (SM CV_1-4) | smoothed, 0.0-1.0 |
/// | `cv[0..4]` | panel CV jacks 1-4 (SM CV_5-8) | smoothed, inversion-corrected, -1.0..+1.0 (±5 V), no deadzone |
/// | `gate1` / `gate2` | Gate In 1/2 | raw level, undebounced (minimum trigger latency) |
/// | `button` | B7 momentary | debounced |
/// | `switch` | B8 toggle | debounced |
///
/// The main loop also applies the global [`outputs()`](crate::outputs) store to
/// hardware each control tick (~1 kHz): LED brightness (DAC), the CV OUT jack
/// (DAC), and both gate outputs. Drive them from `process`:
///
/// ```ignore
/// bbx_daisy::outputs().set_led(self.envelope);
/// ```
///
/// Same macro name as the Pod variant; only one is compiled because exactly one
/// board feature is ever active.
#[cfg(feature = "patch_sm")]
#[macro_export]
macro_rules! bbx_daisy_audio_with_controls {
    ($processor_type:ty, $processor_init:expr) => {
        use $crate::__internal::panic_halt as _;

        static mut __BBX_PROCESSOR: core::mem::MaybeUninit<$processor_type> = core::mem::MaybeUninit::uninit();
        static __BBX_CONTROLS: $crate::controls::AtomicControls = $crate::controls::AtomicControls::new();

        fn __bbx_audio_callback(
            input: &$crate::FrameBuffer<{ $crate::audio::BLOCK_SIZE }>,
            output: &mut $crate::FrameBuffer<{ $crate::audio::BLOCK_SIZE }>,
        ) {
            // Race-free snapshot of the control state (main loop is the writer).
            let controls = __BBX_CONTROLS.load();

            unsafe {
                let processor = __BBX_PROCESSOR.assume_init_mut();
                $crate::AudioProcessor::process(processor, input, output, &controls);
            }
        }

        #[$crate::__internal::entry]
        fn main() -> ! {
            unsafe {
                __BBX_PROCESSOR.write($processor_init);
            }

            // Bring up audio plus the whole control surface in one shot
            // (pac::Peripherals::take() is single-use).
            let board = $crate::board::init_audio_with_controls().expect("Failed to initialize audio board");

            // Let the processor precompute sample-rate-dependent state before streaming.
            unsafe {
                let processor = __BBX_PROCESSOR.assume_init_mut();
                $crate::AudioProcessor::prepare(processor, $crate::audio::DEFAULT_SAMPLE_RATE);
            }

            // Set the audio callback.
            $crate::audio::set_callback(__bbx_audio_callback).expect("audio already running");

            let $crate::board::AudioBoardWithControls {
                audio,
                mut adc1,
                mut knob1_pin,
                mut knob2_pin,
                mut knob3_pin,
                mut knob4_pin,
                mut cv1_pin,
                mut cv2_pin,
                mut cv3_pin,
                mut cv4_pin,
                button_pin,
                switch_pin,
                gate1_pin,
                gate2_pin,
                mut gate_out1,
                mut gate_out2,
                mut cv_out,
                mut led,
            } = board;

            // B7/B8 are mechanical contacts against a pull-up: debounced, active-low.
            let mut button = $crate::peripherals::Button::new_active_low(button_pin);
            let mut switch = $crate::peripherals::Button::new_active_low(switch_pin);

            // Gate inputs are clean logic through the module's inverting stage:
            // active-low and deliberately undebounced — debouncing would add
            // ~5 ms of trigger latency.
            let gate1 = $crate::peripherals::GateIn::new_active_low(gate1_pin);
            let gate2 = $crate::peripherals::GateIn::new_active_low(gate2_pin);

            // Smoothing state lives in the main loop — only the atomic stores
            // are shared with the ISR. CV jacks skip the endpoint deadzone so
            // 1V/oct tracking stays linear.
            let mut knob1 = $crate::peripherals::Knob::default_smoothing_const();
            let mut knob2 = $crate::peripherals::Knob::default_smoothing_const();
            let mut knob3 = $crate::peripherals::Knob::default_smoothing_const();
            let mut knob4 = $crate::peripherals::Knob::default_smoothing_const();
            let mut cv1 = $crate::peripherals::Knob::cv_smoothing_const();
            let mut cv2 = $crate::peripherals::Knob::cv_smoothing_const();
            let mut cv3 = $crate::peripherals::Knob::cv_smoothing_const();
            let mut cv4 = $crate::peripherals::Knob::cv_smoothing_const();

            // Start audio processing (consumes audio peripherals).
            $crate::audio::init_and_start(
                audio.sample_rate,
                audio.sai1,
                audio.dma1,
                audio.dma1_rec,
                audio.sai1_pins,
                audio.sai1_rec,
                &audio.clocks,
            )
            .expect("Failed to start audio streaming");

            // Main loop: publish inputs to the ISR, apply processor outputs to
            // hardware. Wakes on the audio DMA IRQ, so it ticks at ~1 kHz.
            loop {
                // ADC returns u32; shift down to 12-bit range for processing.
                // A failed read falls back to mid-scale ≈ knob center / 0 V,
                // never full-scale.
                let raw_k1 = (adc1.read(&mut knob1_pin).unwrap_or(2048_u32 << 4) >> 4) as u16;
                let raw_k2 = (adc1.read(&mut knob2_pin).unwrap_or(2048_u32 << 4) >> 4) as u16;
                let raw_k3 = (adc1.read(&mut knob3_pin).unwrap_or(2048_u32 << 4) >> 4) as u16;
                let raw_k4 = (adc1.read(&mut knob4_pin).unwrap_or(2048_u32 << 4) >> 4) as u16;
                let raw_c1 = (adc1.read(&mut cv1_pin).unwrap_or(2048_u32 << 4) >> 4) as u16;
                let raw_c2 = (adc1.read(&mut cv2_pin).unwrap_or(2048_u32 << 4) >> 4) as u16;
                let raw_c3 = (adc1.read(&mut cv3_pin).unwrap_or(2048_u32 << 4) >> 4) as u16;
                let raw_c4 = (adc1.read(&mut cv4_pin).unwrap_or(2048_u32 << 4) >> 4) as u16;

                __BBX_CONTROLS.set_knob(0, knob1.process_u12(raw_k1));
                __BBX_CONTROLS.set_knob(1, knob2.process_u12(raw_k2));
                __BBX_CONTROLS.set_knob(2, knob3.process_u12(raw_k3));
                __BBX_CONTROLS.set_knob(3, knob4.process_u12(raw_k4));
                __BBX_CONTROLS.set_cv(
                    0,
                    $crate::peripherals::adc::patch_sm_bipolar(cv1.process_u12(raw_c1)),
                );
                __BBX_CONTROLS.set_cv(
                    1,
                    $crate::peripherals::adc::patch_sm_bipolar(cv2.process_u12(raw_c2)),
                );
                __BBX_CONTROLS.set_cv(
                    2,
                    $crate::peripherals::adc::patch_sm_bipolar(cv3.process_u12(raw_c3)),
                );
                __BBX_CONTROLS.set_cv(
                    3,
                    $crate::peripherals::adc::patch_sm_bipolar(cv4.process_u12(raw_c4)),
                );
                __BBX_CONTROLS.set_gate1(gate1.is_active());
                __BBX_CONTROLS.set_gate2(gate2.is_active());
                __BBX_CONTROLS.set_button(button.update());
                __BBX_CONTROLS.set_switch(switch.update());

                // Apply processor-driven outputs (LED, CV out, gate outs).
                let outputs = $crate::controls::outputs().load();
                led.set_norm(outputs.led);
                cv_out.set_norm(outputs.cv_out);
                gate_out1.set(outputs.gate_out1);
                gate_out2.set(outputs.gate_out2);

                $crate::__internal::wfi();
            }
        }
    };
}

/// Entry point macro for general (non-audio) applications.
///
/// This macro creates a complete entry point for Daisy applications that
/// use GPIO, ADC, or other peripherals without audio processing.
///
/// # Usage
///
/// Pass a function that takes a [`Board`](crate::Board) and returns `!`:
///
/// ```ignore
/// #![no_std]
/// #![no_main]
///
/// use bbx_daisy::prelude::*;
///
/// fn blink(mut board: Board) -> ! {
///     let mut led = Led::new(board.gpioc.pc7.into_push_pull_output());
///     loop {
///         led.toggle();
///         board.delay.delay_ms(500u32);
///     }
/// }
///
/// bbx_daisy_run!(blink);
/// ```
#[macro_export]
macro_rules! bbx_daisy_run {
    ($main_fn:expr) => {
        use $crate::__internal::panic_halt as _;

        #[$crate::__internal::entry]
        fn main() -> ! {
            let board = $crate::Board::init();
            ($main_fn)(board)
        }
    };
}
