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
/// - ADC initialization for hardware controls (knobs, CVs)
/// - Control value smoothing
///
/// # Hardware Controls
///
/// For Pod: Knob 1 (PC4), Knob 2 (PC1)
/// For Seed: No built-in controls (Controls will be default values)
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
/// struct TunableSine {
///     phase: f32,
/// }
///
/// impl AudioProcessor for TunableSine {
///     fn process(
///         &mut self,
///         _input: &FrameBuffer<BLOCK_SIZE>,
///         output: &mut FrameBuffer<BLOCK_SIZE>,
///         controls: &Controls,
///     ) {
///         // Map knob1 to frequency (110Hz - 880Hz)
///         let frequency = 110.0 + controls.knob1 * 770.0;
///         let phase_inc = frequency / DEFAULT_SAMPLE_RATE;
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
/// bbx_daisy_audio!(TunableSine, TunableSine { phase: 0.0 });
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
/// This variant initializes ADC hardware for reading knobs on Pod hardware.
/// Use this when you need real-time control input during audio processing.
///
/// The knob values are read in the main loop and available in the
/// `controls` parameter of `AudioProcessor::process()`.
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
///         let freq = 110.0 + controls.knob1 * 770.0;
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
                __BBX_CONTROLS.set_knob1(knob1.process_u12(raw1));
                __BBX_CONTROLS.set_knob2(knob2.process_u12(raw2));

                $crate::__internal::wfi();
            }
        }
    };
}

/// Patch.Init (`patch_sm`) variant of [`bbx_daisy_audio_with_controls`].
///
/// Initializes ADC1 for the four CV inputs (CV_1=PC0, CV_2=PA3, CV_3=PB1, CV_4=PA7) and the
/// B8 toggle (PB9, active-low), then in the main loop populates `controls.cv[0..4]` (smoothed)
/// and `controls.switch` for use in [`AudioProcessor::process`](crate::AudioProcessor::process).
///
/// Same macro name as the Pod variant; only one is compiled because exactly one board feature
/// is ever active.
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

            // Initialize the board with ADC for the four CV inputs and the B8 switch.
            let board = $crate::board::init_audio_with_cv().expect("Failed to initialize audio board with CV");

            // Let the processor precompute sample-rate-dependent state before streaming.
            unsafe {
                let processor = __BBX_PROCESSOR.assume_init_mut();
                $crate::AudioProcessor::prepare(processor, $crate::audio::DEFAULT_SAMPLE_RATE);
            }

            // Set the audio callback.
            $crate::audio::set_callback(__bbx_audio_callback).expect("audio already running");

            // Destructure board to extract audio peripherals, ADC, CV pins, and switch.
            let $crate::board::AudioBoardWithCv {
                audio,
                mut adc1,
                mut cv1_pin,
                mut cv2_pin,
                mut cv3_pin,
                mut cv4_pin,
                switch_pin,
            } = board;

            // Wrap the B8 toggle in a debounced, active-low button.
            let mut switch = $crate::peripherals::Button::new_active_low(switch_pin);

            // CV smoothing state lives in the main loop — only the atomic
            // controls store is shared with the ISR.
            let mut cv1 = $crate::peripherals::Knob::default_smoothing_const();
            let mut cv2 = $crate::peripherals::Knob::default_smoothing_const();
            let mut cv3 = $crate::peripherals::Knob::default_smoothing_const();
            let mut cv4 = $crate::peripherals::Knob::default_smoothing_const();

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

            // Main loop: read CVs + switch and update controls.
            loop {
                // ADC returns u32; shift down to 12-bit range for processing.
                let raw1 = (adc1.read(&mut cv1_pin).unwrap_or(0_u32) >> 4) as u16;
                let raw2 = (adc1.read(&mut cv2_pin).unwrap_or(0_u32) >> 4) as u16;
                let raw3 = (adc1.read(&mut cv3_pin).unwrap_or(0_u32) >> 4) as u16;
                let raw4 = (adc1.read(&mut cv4_pin).unwrap_or(0_u32) >> 4) as u16;
                let switch_state = switch.update();

                // Smooth and publish to the ISR-visible atomic store.
                __BBX_CONTROLS.set_cv(0, cv1.process_u12(raw1));
                __BBX_CONTROLS.set_cv(1, cv2.process_u12(raw2));
                __BBX_CONTROLS.set_cv(2, cv3.process_u12(raw3));
                __BBX_CONTROLS.set_cv(3, cv4.process_u12(raw4));
                __BBX_CONTROLS.set_switch(switch_state);

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
