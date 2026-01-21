//! Entry point macros for Daisy applications.
//!
//! These macros eliminate boilerplate by handling:
//! - Entry point setup (`#[cortex_m_rt::entry]`)
//! - Panic handler (`panic_halt`)
//! - Audio callback registration
//! - Main loop with `wfi()`

/// Entry point macro for audio processing applications.
///
/// This macro creates a complete entry point for Daisy audio applications.
/// It handles all unsafe static state management internally, so users never
/// need to write `unsafe` code.
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
/// struct SineOsc {
///     phase: f32,
///     phase_inc: f32,
/// }
///
/// impl AudioProcessor for SineOsc {
///     fn process(&mut self, _input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
///         for i in 0..BLOCK_SIZE {
///             let sample = sinf(self.phase * 2.0 * PI) * 0.5;
///             output.set_frame(i, sample, sample);
///             self.phase += self.phase_inc;
///             if self.phase >= 1.0 {
///                 self.phase -= 1.0;
///             }
///         }
///     }
/// }
///
/// bbx_daisy_audio!(SineOsc, SineOsc::new(440.0));
/// ```
#[macro_export]
macro_rules! bbx_daisy_audio {
    ($processor_type:ty, $processor_init:expr) => {
        use $crate::__internal::panic_halt as _;

        static mut __BBX_PROCESSOR: core::mem::MaybeUninit<$processor_type> = core::mem::MaybeUninit::uninit();

        fn __bbx_audio_callback(
            input: &$crate::FrameBuffer<{ $crate::audio::BLOCK_SIZE }>,
            output: &mut $crate::FrameBuffer<{ $crate::audio::BLOCK_SIZE }>,
        ) {
            unsafe {
                let processor = __BBX_PROCESSOR.assume_init_mut();
                $crate::AudioProcessor::process(processor, input, output);
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
