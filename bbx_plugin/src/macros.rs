//! FFI export macro for plugin DSP implementations.
//!
//! This module provides the `bbx_plugin_ffi!` macro that generates
//! all C FFI exports for a given `PluginDsp` implementation.

/// Generate C FFI exports for a `PluginDsp` implementation.
///
/// This macro generates the following FFI functions:
/// - `bbx_graph_create()` - Create a new DSP graph
/// - `bbx_graph_destroy()` - Destroy a DSP graph
/// - `bbx_graph_prepare()` - Prepare for playback
/// - `bbx_graph_reset()` - Reset DSP state
/// - `bbx_graph_process()` - Process audio
///
/// # Example
///
/// ```ignore
/// use bbx_dsp::PluginDsp;
/// use bbx_ffi::bbx_plugin_ffi;
///
/// pub struct PluginGraph { /* ... */ }
/// impl PluginDsp for PluginGraph { /* ... */ }
///
/// // Generate all FFI exports
/// bbx_plugin_ffi!(PluginGraph);
/// ```
#[macro_export]
macro_rules! bbx_plugin_ffi {
    ($dsp_type:ty) => {
        // Type alias for this plugin's graph
        type PluginGraphInner = $crate::GraphInner<$dsp_type>;

        /// Create a new DSP effects chain.
        #[unsafe(no_mangle)]
        pub extern "C" fn bbx_graph_create() -> *mut $crate::BbxGraph {
            let inner = Box::new(PluginGraphInner::new());
            $crate::handle_from_graph(inner)
        }

        /// Destroy a DSP effects chain.
        #[unsafe(no_mangle)]
        pub extern "C" fn bbx_graph_destroy(handle: *mut $crate::BbxGraph) {
            if !handle.is_null() {
                unsafe {
                    drop(Box::from_raw(handle as *mut PluginGraphInner));
                }
            }
        }

        /// Prepare the effects chain for playback.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn bbx_graph_prepare(
            handle: *mut $crate::BbxGraph,
            sample_rate: f64,
            buffer_size: u32,
            num_channels: u32,
        ) -> $crate::BbxError {
            if handle.is_null() {
                return $crate::BbxError::NullPointer;
            }
            if buffer_size == 0 {
                return $crate::BbxError::InvalidBufferSize;
            }
            if num_channels == 0 {
                return $crate::BbxError::InvalidParameter;
            }

            unsafe {
                let inner = $crate::graph_from_handle::<$dsp_type>(handle);
                inner.prepare(sample_rate, buffer_size as usize, num_channels as usize);
            }

            $crate::BbxError::Ok
        }

        /// Reset the effects chain state.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn bbx_graph_reset(handle: *mut $crate::BbxGraph) -> $crate::BbxError {
            if handle.is_null() {
                return $crate::BbxError::NullPointer;
            }

            unsafe {
                let inner = $crate::graph_from_handle::<$dsp_type>(handle);
                inner.reset();
            }

            $crate::BbxError::Ok
        }

        /// Process a block of audio through the effects chain.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn bbx_graph_process(
            handle: *mut $crate::BbxGraph,
            inputs: *const *const f32,
            outputs: *mut *mut f32,
            num_channels: u32,
            num_samples: u32,
            params: *const f32,
            num_params: u32,
            midi_events: *const $crate::midi::MidiEvent,
            num_midi_events: u32,
        ) {
            $crate::process_audio::<$dsp_type>(
                handle,
                inputs,
                outputs,
                num_channels,
                num_samples,
                params,
                num_params,
                midi_events,
                num_midi_events,
            );
        }
    };
}
