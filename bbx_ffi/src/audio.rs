//! Audio processing FFI functions.
//!
//! This module provides the main audio processing function that bridges
//! JUCE's processBlock to the Rust DSP graph.

use crate::handle::{BbxGraph, GraphInner, graph_from_handle};

/// Process a block of audio.
///
/// # Parameters
///
/// - `handle`: Valid graph handle created by `bbx_graph_create`.
/// - `inputs`: Array of input channel pointers (can be null for synths).
/// - `outputs`: Array of output channel pointers.
/// - `num_channels`: Number of audio channels.
/// - `num_samples`: Number of samples per channel in this block.
/// - `params`: Pointer to flat float array of parameter values.
/// - `num_params`: Number of parameters in the array.
///
/// # Safety
///
/// - `handle` must be a valid pointer from `bbx_graph_create`.
/// - `outputs` must be a valid pointer to an array of `num_channels` pointers.
/// - Each output channel pointer must be valid for `num_samples` floats.
/// - `params` must be valid for `num_params` floats, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bbx_graph_process(
    handle: *mut BbxGraph,
    _inputs: *const *const f32,
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

        // Process MIDI events and update voice state
        process_midi_events(inner);

        // Create output buffer slices
        // Using a fixed-size array on the stack for typical channel counts
        const MAX_CHANNELS: usize = 8;
        let mut output_slices: [Option<&mut [f32]>; MAX_CHANNELS] = Default::default();

        for (i, slot) in output_slices.iter_mut().enumerate().take(num_channels.min(MAX_CHANNELS)) {
            let ptr = *outputs.add(i);
            if !ptr.is_null() {
                *slot = Some(std::slice::from_raw_parts_mut(ptr, num_samples));
            }
        }

        // Convert to the format Graph expects
        let mut refs: Vec<&mut [f32]> = output_slices
            .iter_mut()
            .take(num_channels)
            .filter_map(|opt| opt.as_mut().map(|s| &mut **s))
            .collect();

        // Process the graph
        inner.graph.process_buffers(&mut refs);

        // Clear MIDI buffer for next block
        inner.midi_buffer.clear();
    }
}

/// Apply parameter values from the flat array to the graph blocks.
fn apply_parameters(inner: &mut GraphInner, params: &[f32]) {
    // Update the param cache
    inner.param_cache.clear();
    inner.param_cache.extend_from_slice(params);

    // TODO: Apply parameters to specific blocks
    // This will be implemented when we add parameter binding to blocks
    // For now, parameters are cached for later use
}

/// Process accumulated MIDI events and update voice state.
fn process_midi_events(inner: &mut GraphInner) {
    use bbx_midi::MidiMessageStatus;

    for msg in inner.midi_buffer.iter() {
        match msg.get_status() {
            MidiMessageStatus::NoteOn => {
                if let (Some(note), Some(velocity)) = (msg.get_note_number(), msg.get_velocity()) {
                    if velocity > 0 {
                        inner.voice_state.note_on(note, velocity);
                        // TODO: Trigger envelope, set oscillator frequency
                    } else {
                        // Note on with velocity 0 is note off
                        inner.voice_state.note_off(note);
                    }
                }
            }
            MidiMessageStatus::NoteOff => {
                if let Some(note) = msg.get_note_number() {
                    inner.voice_state.note_off(note);
                    // TODO: Trigger envelope release
                }
            }
            _ => {
                // Handle other MIDI messages (CC, pitch wheel, etc.) as needed
            }
        }
    }
}
