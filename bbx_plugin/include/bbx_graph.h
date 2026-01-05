/* bbx_graph - C++ RAII wrapper for bbx_audio DSP library */

#pragma once

#include "bbx_ffi.h"

namespace bbx {

/**
 * RAII wrapper for the bbx_plugin C API.
 *
 * Manages the lifecycle of a BbxGraph handle and provides
 * a C++ interface for audio processing.
 *
 * Header-only for easy inclusion - no separate compilation required.
 */
class Graph {
public:
    Graph()
        : m_handle(bbx_graph_create())
    {
    }

    ~Graph()
    {
        if (m_handle) {
            bbx_graph_destroy(m_handle);
        }
    }

    // Non-copyable
    Graph(const Graph&) = delete;
    Graph& operator=(const Graph&) = delete;

    // Movable
    Graph(Graph&& other) noexcept
        : m_handle(other.m_handle)
    {
        other.m_handle = nullptr;
    }

    Graph& operator=(Graph&& other) noexcept
    {
        if (this != &other) {
            if (m_handle) {
                bbx_graph_destroy(m_handle);
            }
            m_handle = other.m_handle;
            other.m_handle = nullptr;
        }
        return *this;
    }

    /**
     * Prepare the DSP graph for playback.
     *
     * @param sampleRate Sample rate in Hz.
     * @param bufferSize Number of samples per buffer.
     * @param numChannels Number of audio channels.
     * @return BBX_ERROR_OK on success.
     */
    BbxError Prepare(double sampleRate, uint32_t bufferSize, uint32_t numChannels)
    {
        if (!m_handle) {
            return BBX_ERROR_NULL_POINTER;
        }
        return bbx_graph_prepare(m_handle, sampleRate, bufferSize, numChannels);
    }

    /**
     * Reset the DSP graph state.
     *
     * @return BBX_ERROR_OK on success.
     */
    BbxError Reset()
    {
        if (!m_handle) {
            return BBX_ERROR_NULL_POINTER;
        }
        return bbx_graph_reset(m_handle);
    }

    /**
     * Process a block of audio through the DSP graph.
     *
     * @param inputs Array of input channel pointers.
     * @param outputs Array of output channel pointers.
     * @param numChannels Number of audio channels.
     * @param numSamples Number of samples per channel.
     * @param params Pointer to parameter array.
     * @param numParams Number of parameters.
     * @param midiEvents Pointer to array of MIDI events (may be nullptr for effects).
     * @param numMidiEvents Number of MIDI events in the array.
     */
    void Process(const float* const* inputs,
        float* const* outputs,
        uint32_t numChannels,
        uint32_t numSamples,
        const float* params,
        uint32_t numParams,
        const BbxMidiEvent* midiEvents = nullptr,
        uint32_t numMidiEvents = 0)
    {
        if (m_handle) {
            bbx_graph_process(m_handle, inputs, outputs, numChannels, numSamples, params, numParams, midiEvents, numMidiEvents);
        }
    }

    /**
     * Check if the graph holds a valid handle.
     */
    bool IsValid() const { return m_handle != nullptr; }

    /**
     * Access the raw handle for advanced use.
     */
    BbxGraph* handle() { return m_handle; }
    const BbxGraph* handle() const { return m_handle; }

private:
    BbxGraph* m_handle { nullptr };
};

} // namespace bbx
