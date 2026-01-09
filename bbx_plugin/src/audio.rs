//! Audio processing FFI functions.
//!
//! This module provides the generic audio processing function that bridges
//! JUCE's processBlock to the Rust effects chain using zero-copy buffer handling.

use std::panic::{AssertUnwindSafe, catch_unwind};

use bbx_dsp::PluginDsp;
use bbx_midi::MidiEvent;

use crate::handle::{BbxGraph, graph_from_handle};

/// Maximum samples per buffer.
///
/// Buffers larger than this are processed up to this limit only.
/// This value accommodates most audio workflows (e.g., 4096 samples at 192kHz = ~21ms).
/// Configure your DAW to use buffer sizes <= 4096 for full coverage.
const MAX_SAMPLES: usize = 4096;

/// Maximum channels supported.
const MAX_CHANNELS: usize = 2;

/// Process a block of audio through the effects chain.
///
/// This function is called by the macro-generated `bbx_graph_process` FFI function.
/// It uses zero-copy buffer handling for optimal performance.
///
/// # Safety
///
/// - `handle` must be a valid pointer from `bbx_graph_create`.
/// - `inputs` must be a valid pointer to an array of `num_channels` pointers, or null.
/// - `outputs` must be a valid pointer to an array of `num_channels` pointers.
/// - Each input/output channel pointer must be valid for `num_samples` floats.
/// - `params` must be valid for `num_params` floats, or null.
/// - `midi_events` must be valid for `num_midi_events` MidiEvent structs, or null.
#[allow(clippy::too_many_arguments)]
pub unsafe fn process_audio<D: PluginDsp>(
    handle: *mut BbxGraph,
    inputs: *const *const f32,
    outputs: *mut *mut f32,
    num_channels: u32,
    num_samples: u32,
    params: *const f32,
    num_params: u32,
    midi_events: *const MidiEvent,
    num_midi_events: u32,
) {
    if handle.is_null() || outputs.is_null() {
        return;
    }

    let result = catch_unwind(AssertUnwindSafe(|| unsafe {
        let inner = graph_from_handle::<D>(handle);

        let num_channels = num_channels as usize;
        let num_samples = num_samples as usize;

        if !inner.prepared {
            for i in 0..num_channels {
                let output_ptr = *outputs.add(i);
                if !output_ptr.is_null() {
                    std::ptr::write_bytes(output_ptr, 0, num_samples * size_of::<f32>());
                }
            }
            return;
        }

        // Debug check for buffer size limits
        debug_assert!(
            num_samples <= MAX_SAMPLES,
            "Buffer size {num_samples} exceeds MAX_SAMPLES ({MAX_SAMPLES}). Only first {MAX_SAMPLES} samples processed."
        );
        debug_assert!(
            num_channels <= MAX_CHANNELS,
            "Channel count {num_channels} exceeds MAX_CHANNELS ({MAX_CHANNELS}). Only first {MAX_CHANNELS} channels processed."
        );

        let samples_to_process = num_samples.min(MAX_SAMPLES);
        let channels_to_process = num_channels.min(MAX_CHANNELS);

        // Apply parameters if provided
        if !params.is_null() && num_params > 0 {
            let param_slice = std::slice::from_raw_parts(params, num_params as usize);
            inner.dsp.apply_parameters(param_slice);
        }

        // Build MIDI slice (empty if null pointer or zero count)
        let midi_slice: &[MidiEvent] = if !midi_events.is_null() && num_midi_events > 0 {
            std::slice::from_raw_parts(midi_events, num_midi_events as usize)
        } else {
            &[]
        };

        // Build input slices directly from FFI pointers (zero-copy)
        // Use a small stack buffer for silent channels when input is null
        let silent_buffer: [f32; MAX_SAMPLES] = [0.0; MAX_SAMPLES];
        let silent_slice: &[f32] = &silent_buffer[..samples_to_process];

        let mut input_slices_storage: [&[f32]; MAX_CHANNELS] = [silent_slice; MAX_CHANNELS];

        if !inputs.is_null() {
            #[allow(clippy::needless_range_loop)] // ch is used for both array indexing and pointer arithmetic
            for ch in 0..channels_to_process {
                let input_ptr = *inputs.add(ch);
                if !input_ptr.is_null() {
                    input_slices_storage[ch] = std::slice::from_raw_parts(input_ptr, samples_to_process);
                }
            }
        }

        // Build output slices directly from FFI pointers (zero-copy)
        // JUCE guarantees valid output pointers for all requested channels
        let output_ptr_0 = *outputs.add(0);
        let output_ptr_1 = if channels_to_process > 1 {
            *outputs.add(1)
        } else {
            output_ptr_0
        };

        if output_ptr_0.is_null() {
            return;
        }

        let output_slice_0 = std::slice::from_raw_parts_mut(output_ptr_0, samples_to_process);

        if channels_to_process == 1 {
            let mut output_refs: [&mut [f32]; 1] = [output_slice_0];
            inner
                .dsp
                .process(&input_slices_storage[..1], &mut output_refs, midi_slice, &inner.context);
        } else {
            if output_ptr_1.is_null() {
                return;
            }
            let output_slice_1 = std::slice::from_raw_parts_mut(output_ptr_1, samples_to_process);
            let mut output_refs: [&mut [f32]; 2] = [output_slice_0, output_slice_1];
            inner.dsp.process(
                &input_slices_storage[..channels_to_process],
                &mut output_refs,
                midi_slice,
                &inner.context,
            );
        }
    }));

    // On panic, zero all outputs to produce silence instead of crashing the host
    if result.is_err() {
        unsafe {
            for i in 0..num_channels as usize {
                let output_ptr = *outputs.add(i);
                if !output_ptr.is_null() {
                    std::ptr::write_bytes(output_ptr, 0, num_samples as usize * size_of::<f32>());
                }
            }
        }
    }
}
