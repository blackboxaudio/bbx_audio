//! Audio processing FFI functions.
//!
//! This module provides the main audio processing function that bridges
//! JUCE's processBlock to the Rust effects chain.

use bbx_dsp::{
    block::Block,
    blocks::effectors::channel_router::ChannelMode,
    parameter::Parameter,
};

use crate::{
    handle::{BbxGraph, GraphInner, graph_from_handle},
    params::{PARAM_CHANNEL_MODE, PARAM_DC_OFFSET, PARAM_GAIN, PARAM_INVERT_LEFT, PARAM_INVERT_RIGHT, PARAM_MONO, PARAM_PAN},
};

/// Process a block of audio through the effects chain.
///
/// # Parameters
///
/// - `handle`: Valid graph handle created by `bbx_graph_create`.
/// - `inputs`: Array of input channel pointers.
/// - `outputs`: Array of output channel pointers.
/// - `num_channels`: Number of audio channels.
/// - `num_samples`: Number of samples per channel in this block.
/// - `params`: Pointer to flat float array of parameter values.
/// - `num_params`: Number of parameters in the array.
///
/// # Safety
///
/// - `handle` must be a valid pointer from `bbx_graph_create`.
/// - `inputs` must be a valid pointer to an array of `num_channels` pointers.
/// - `outputs` must be a valid pointer to an array of `num_channels` pointers.
/// - Each input/output channel pointer must be valid for `num_samples` floats.
/// - `params` must be valid for `num_params` floats, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bbx_graph_process(
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
        let inner = graph_from_handle(handle);

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

        // Apply parameters if provided
        if !params.is_null() && num_params > 0 {
            let param_slice = std::slice::from_raw_parts(params, num_params as usize);
            apply_parameters(inner, param_slice);
        }

        // Process effects chain: DC blocker → Channel router → Panner → Gain
        process_effects_chain(inner, inputs, outputs, num_channels, num_samples);
    }
}

/// Apply parameter values from the flat array to the effect blocks.
fn apply_parameters(inner: &mut GraphInner, params: &[f32]) {
    // DC Offset (enable/disable)
    if params.len() > PARAM_DC_OFFSET {
        inner.dc_blocker.enabled = params[PARAM_DC_OFFSET] >= 0.5;
    }

    // Channel Router
    if params.len() > PARAM_INVERT_LEFT {
        inner.channel_router.invert_left = params[PARAM_INVERT_LEFT] >= 0.5;
    }
    if params.len() > PARAM_INVERT_RIGHT {
        inner.channel_router.invert_right = params[PARAM_INVERT_RIGHT] >= 0.5;
    }
    if params.len() > PARAM_CHANNEL_MODE {
        inner.channel_router.mode = ChannelMode::from(params[PARAM_CHANNEL_MODE]);
    }
    if params.len() > PARAM_MONO {
        inner.channel_router.mono = params[PARAM_MONO] >= 0.5;
    }

    // Panner
    if params.len() > PARAM_PAN {
        inner.panner.position = Parameter::Constant(params[PARAM_PAN]);
    }

    // Gain
    if params.len() > PARAM_GAIN {
        inner.gain.level_db = Parameter::Constant(params[PARAM_GAIN]);
    }
}

/// Process the effects chain: input → DC blocker → Channel router → Panner → Gain → output.
unsafe fn process_effects_chain(
    inner: &mut GraphInner,
    inputs: *const *const f32,
    outputs: *mut *mut f32,
    num_channels: usize,
    num_samples: usize,
) {
    // Allocate intermediate buffers on stack for typical stereo case
    const MAX_SAMPLES: usize = 4096;
    const MAX_CHANNELS: usize = 2;

    // We'll process in-place using the output buffers as working space
    // First, copy input to output
    if !inputs.is_null() {
        for ch in 0..num_channels.min(MAX_CHANNELS) {
            let input_ptr = unsafe { *inputs.add(ch) };
            let output_ptr = unsafe { *outputs.add(ch) };
            if !input_ptr.is_null() && !output_ptr.is_null() {
                unsafe {
                    std::ptr::copy_nonoverlapping(input_ptr, output_ptr, num_samples);
                }
            }
        }
    }

    // Use stack buffers for intermediate results
    let mut buffer_l: [f32; MAX_SAMPLES] = [0.0; MAX_SAMPLES];
    let mut buffer_r: [f32; MAX_SAMPLES] = [0.0; MAX_SAMPLES];
    let samples_to_process = num_samples.min(MAX_SAMPLES);

    // Copy input to our working buffers
    for ch in 0..num_channels.min(MAX_CHANNELS) {
        let output_ptr = unsafe { *outputs.add(ch) };
        if !output_ptr.is_null() {
            let input_slice = unsafe { std::slice::from_raw_parts(output_ptr, num_samples) };
            let dest = if ch == 0 { &mut buffer_l } else { &mut buffer_r };
            dest[..samples_to_process].copy_from_slice(&input_slice[..samples_to_process]);
        }
    }

    // Process chain: Each block takes input and writes to output
    let empty_modulation: [f32; 0] = [];

    // 1. DC Blocker
    {
        let inputs: [&[f32]; 2] = [&buffer_l[..samples_to_process], &buffer_r[..samples_to_process]];
        let mut temp_l: [f32; MAX_SAMPLES] = [0.0; MAX_SAMPLES];
        let mut temp_r: [f32; MAX_SAMPLES] = [0.0; MAX_SAMPLES];
        let mut outputs: [&mut [f32]; 2] = [&mut temp_l[..samples_to_process], &mut temp_r[..samples_to_process]];

        inner.dc_blocker.process(&inputs, &mut outputs, &empty_modulation, &inner.context);

        buffer_l[..samples_to_process].copy_from_slice(&temp_l[..samples_to_process]);
        buffer_r[..samples_to_process].copy_from_slice(&temp_r[..samples_to_process]);
    }

    // 2. Channel Router
    {
        let inputs: [&[f32]; 2] = [&buffer_l[..samples_to_process], &buffer_r[..samples_to_process]];
        let mut temp_l: [f32; MAX_SAMPLES] = [0.0; MAX_SAMPLES];
        let mut temp_r: [f32; MAX_SAMPLES] = [0.0; MAX_SAMPLES];
        let mut outputs: [&mut [f32]; 2] = [&mut temp_l[..samples_to_process], &mut temp_r[..samples_to_process]];

        inner.channel_router.process(&inputs, &mut outputs, &empty_modulation, &inner.context);

        buffer_l[..samples_to_process].copy_from_slice(&temp_l[..samples_to_process]);
        buffer_r[..samples_to_process].copy_from_slice(&temp_r[..samples_to_process]);
    }

    // 3. Panner
    {
        let inputs: [&[f32]; 2] = [&buffer_l[..samples_to_process], &buffer_r[..samples_to_process]];
        let mut temp_l: [f32; MAX_SAMPLES] = [0.0; MAX_SAMPLES];
        let mut temp_r: [f32; MAX_SAMPLES] = [0.0; MAX_SAMPLES];
        let mut outputs: [&mut [f32]; 2] = [&mut temp_l[..samples_to_process], &mut temp_r[..samples_to_process]];

        inner.panner.process(&inputs, &mut outputs, &empty_modulation, &inner.context);

        buffer_l[..samples_to_process].copy_from_slice(&temp_l[..samples_to_process]);
        buffer_r[..samples_to_process].copy_from_slice(&temp_r[..samples_to_process]);
    }

    // 4. Gain
    {
        let inputs: [&[f32]; 2] = [&buffer_l[..samples_to_process], &buffer_r[..samples_to_process]];
        let mut temp_l: [f32; MAX_SAMPLES] = [0.0; MAX_SAMPLES];
        let mut temp_r: [f32; MAX_SAMPLES] = [0.0; MAX_SAMPLES];
        let mut outputs: [&mut [f32]; 2] = [&mut temp_l[..samples_to_process], &mut temp_r[..samples_to_process]];

        inner.gain.process(&inputs, &mut outputs, &empty_modulation, &inner.context);

        buffer_l[..samples_to_process].copy_from_slice(&temp_l[..samples_to_process]);
        buffer_r[..samples_to_process].copy_from_slice(&temp_r[..samples_to_process]);
    }

    // Copy results back to output buffers
    for ch in 0..num_channels.min(MAX_CHANNELS) {
        let output_ptr = unsafe { *outputs.add(ch) };
        if !output_ptr.is_null() {
            let output_slice = unsafe { std::slice::from_raw_parts_mut(output_ptr, num_samples) };
            let src = if ch == 0 { &buffer_l } else { &buffer_r };
            output_slice[..samples_to_process].copy_from_slice(&src[..samples_to_process]);
        }
    }
}
