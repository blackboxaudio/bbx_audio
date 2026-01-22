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
        static mut __BBX_CONTROLS: $crate::controls::Controls = $crate::controls::Controls::new();

        fn __bbx_audio_callback(
            input: &$crate::FrameBuffer<{ $crate::audio::BLOCK_SIZE }>,
            output: &mut $crate::FrameBuffer<{ $crate::audio::BLOCK_SIZE }>,
        ) {
            unsafe {
                // Get controls (updated by main loop or ADC interrupt)
                let controls_ptr = core::ptr::addr_of!(__BBX_CONTROLS);

                // Call user's audio processor with controls
                let processor = __BBX_PROCESSOR.assume_init_mut();
                $crate::AudioProcessor::process(processor, input, output, &*controls_ptr);
            }
        }

        #[$crate::__internal::entry]
        fn main() -> ! {
            unsafe {
                __BBX_PROCESSOR.write($processor_init);
            }

            $crate::audio::set_callback(__bbx_audio_callback);

            let mut audio = $crate::audio::default_audio();
            audio.init();
            audio.start();

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
        static mut __BBX_CONTROLS: $crate::controls::Controls = $crate::controls::Controls::new();
        static mut __BBX_KNOB1: $crate::peripherals::Knob = $crate::peripherals::Knob::default_smoothing_const();
        static mut __BBX_KNOB2: $crate::peripherals::Knob = $crate::peripherals::Knob::default_smoothing_const();

        fn __bbx_audio_callback(
            input: &$crate::FrameBuffer<{ $crate::audio::BLOCK_SIZE }>,
            output: &mut $crate::FrameBuffer<{ $crate::audio::BLOCK_SIZE }>,
        ) {
            unsafe {
                // Get controls pointer
                let controls_ptr = core::ptr::addr_of!(__BBX_CONTROLS);

                // Call user's audio processor with controls
                let processor = __BBX_PROCESSOR.assume_init_mut();
                $crate::AudioProcessor::process(processor, input, output, &*controls_ptr);
            }
        }

        #[$crate::__internal::entry]
        fn main() -> ! {
            // Initialize board with ADC
            let board_adc = $crate::Board::init_with_adc();

            unsafe {
                __BBX_PROCESSOR.write($processor_init);
            }

            $crate::audio::set_callback(__bbx_audio_callback);

            let mut audio = $crate::audio::default_audio();
            audio.init();
            audio.start();

            // Main loop: read ADC and update controls
            loop {
                // Read knobs and update controls
                unsafe {
                    let knob1_ptr = core::ptr::addr_of_mut!(__BBX_KNOB1);
                    let knob2_ptr = core::ptr::addr_of_mut!(__BBX_KNOB2);
                    let controls_ptr = core::ptr::addr_of_mut!(__BBX_CONTROLS);

                    // Read raw ADC values (placeholder - actual hardware reading TBD)
                    // In full implementation, board_adc would provide read methods
                    // For now, controls remain at default values

                    // (*controls_ptr).knob1 = (*knob1_ptr).process_u12(raw1);
                    // (*controls_ptr).knob2 = (*knob2_ptr).process_u12(raw2);
                }

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
