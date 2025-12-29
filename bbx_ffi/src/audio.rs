//! Audio processing FFI functions.
//!
//! This module provides the generic audio processing function that bridges
//! JUCE's processBlock to the Rust effects chain.

use bbx_dsp::PluginDsp;

use crate::handle::{BbxGraph, graph_from_handle};

/// Maximum samples per buffer (stack allocation limit).
const MAX_SAMPLES: usize = 4096;

/// Maximum channels supported.
const MAX_CHANNELS: usize = 2;

/// Process a block of audio through the effects chain.
///
/// This function is called by the macro-generated `bbx_graph_process` FFI function.
///
/// # Safety
///
/// - `handle` must be a valid pointer from `bbx_graph_create`.
/// - `inputs` must be a valid pointer to an array of `num_channels` pointers.
/// - `outputs` must be a valid pointer to an array of `num_channels` pointers.
/// - Each input/output channel pointer must be valid for `num_samples` floats.
/// - `params` must be valid for `num_params` floats, or null.
pub unsafe fn process_audio<D: PluginDsp>(
    handle: *mut BbxGraph,
    inputs: *const *const f32,
    outputs: *mut *mut f32,
    num_channels: u32,
    num_samples: u32,
    params: *const f32,
    num_params: u32,
) {
    if handle.is_null() || outputs.is_null() {
        return;
    }

    unsafe {
        let inner = graph_from_handle::<D>(handle);

        if !inner.prepared {
            // Graph not prepared, output silence
            for i in 0..num_channels as usize {
                let output_ptr = *outputs.add(i);
                if !output_ptr.is_null() {
                    std::ptr::write_bytes(output_ptr, 0, num_samples as usize);
                }
            }
            return;
        }

        let num_channels = num_channels as usize;
        let num_samples = num_samples as usize;
        let samples_to_process = num_samples.min(MAX_SAMPLES);

        // Apply parameters if provided
        if !params.is_null() && num_params > 0 {
            let param_slice = std::slice::from_raw_parts(params, num_params as usize);
            inner.dsp.apply_parameters(param_slice);
        }

        // Build input slices
        let mut input_buffers: [[f32; MAX_SAMPLES]; MAX_CHANNELS] = [[0.0; MAX_SAMPLES]; MAX_CHANNELS];
        if !inputs.is_null() {
            for ch in 0..num_channels.min(MAX_CHANNELS) {
                let input_ptr = *inputs.add(ch);
                if !input_ptr.is_null() {
                    let src = std::slice::from_raw_parts(input_ptr, samples_to_process);
                    input_buffers[ch][..samples_to_process].copy_from_slice(src);
                }
            }
        }

        // Build output buffers
        let mut output_buffers: [[f32; MAX_SAMPLES]; MAX_CHANNELS] = [[0.0; MAX_SAMPLES]; MAX_CHANNELS];

        // Create slice references for the trait method
        let input_slices: [&[f32]; MAX_CHANNELS] = [
            &input_buffers[0][..samples_to_process],
            &input_buffers[1][..samples_to_process],
        ];

        // Split the output buffers to satisfy borrow checker
        let (left_buf, rest) = output_buffers.split_at_mut(1);
        let (right_buf, _) = rest.split_at_mut(1);
        let mut output_slices: [&mut [f32]; MAX_CHANNELS] = [
            &mut left_buf[0][..samples_to_process],
            &mut right_buf[0][..samples_to_process],
        ];

        // Call the plugin's process method
        inner.dsp.process(
            &input_slices[..num_channels.min(MAX_CHANNELS)],
            &mut output_slices[..num_channels.min(MAX_CHANNELS)],
            &inner.context,
        );

        // Copy results back to output buffers
        for ch in 0..num_channels.min(MAX_CHANNELS) {
            let output_ptr = *outputs.add(ch);
            if !output_ptr.is_null() {
                let dest = std::slice::from_raw_parts_mut(output_ptr, samples_to_process);
                dest.copy_from_slice(&output_buffers[ch][..samples_to_process]);
            }
        }
    }
}
