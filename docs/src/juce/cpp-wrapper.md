# C++ RAII Wrapper

`bbx_graph.h` provides a header-only C++ wrapper for safe resource management.

## Overview

The `bbx::Graph` class wraps the C FFI with RAII semantics:

- Constructor calls `bbx_graph_create()`
- Destructor calls `bbx_graph_destroy()`
- Move semantics prevent accidental copies

## Usage

Include the header:

```cpp
#include <bbx_graph.h>
```

Create and use a graph:

```cpp
bbx::Graph dsp;  // Automatically creates handle

if (dsp.IsValid()) {
    dsp.Prepare(sampleRate, bufferSize, numChannels);
    dsp.Process(inputs, outputs, numChannels, numSamples, params, numParams);
}
```

## Class Reference

### Constructor

```cpp
Graph();
```

Creates a new DSP graph. Check `IsValid()` to verify allocation succeeded.

### Destructor

```cpp
~Graph();
```

Destroys the graph and frees all resources.

### Move Operations

```cpp
Graph(Graph&& other) noexcept;
Graph& operator=(Graph&& other) noexcept;
```

The class is movable but not copyable:

```cpp
bbx::Graph a;
bbx::Graph b = std::move(a);  // OK - a is now invalid
// bbx::Graph c = b;          // Error - not copyable
```

### Prepare

```cpp
BbxError Prepare(double sampleRate, uint32_t bufferSize, uint32_t numChannels);
```

Prepare for playback. Call from `prepareToPlay()`.

### Reset

```cpp
BbxError Reset();
```

Reset DSP state. Call from `releaseResources()`.

### Process

```cpp
void Process(
    const float* const* inputs,
    float* const* outputs,
    uint32_t numChannels,
    uint32_t numSamples,
    const float* params,
    uint32_t numParams,
    const BbxMidiEvent* midiEvents = nullptr,
    uint32_t numMidiEvents = 0
);
```

Process audio with optional MIDI events. Call from `processBlock()`.

For effects (no MIDI):
```cpp
dsp.Process(inputs, outputs, numChannels, numSamples, params, numParams);
```

For synthesizers (with MIDI):
```cpp
dsp.Process(inputs, outputs, numChannels, numSamples, params, numParams, midiEvents, numMidiEvents);
```

### IsValid

```cpp
bool IsValid() const;
```

Returns `true` if the handle is valid.

### handle

```cpp
BbxGraph* handle();
const BbxGraph* handle() const;
```

Access the raw handle for advanced use.

## Complete Header

```cpp
#pragma once

#include "bbx_ffi.h"

namespace bbx {

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

    BbxError Prepare(double sampleRate, uint32_t bufferSize, uint32_t numChannels)
    {
        if (!m_handle) {
            return BBX_ERROR_NULL_POINTER;
        }
        return bbx_graph_prepare(m_handle, sampleRate, bufferSize, numChannels);
    }

    BbxError Reset()
    {
        if (!m_handle) {
            return BBX_ERROR_NULL_POINTER;
        }
        return bbx_graph_reset(m_handle);
    }

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
            bbx_graph_process(m_handle, inputs, outputs, numChannels, numSamples,
                              params, numParams, midiEvents, numMidiEvents);
        }
    }

    bool IsValid() const { return m_handle != nullptr; }

    BbxGraph* handle() { return m_handle; }
    const BbxGraph* handle() const { return m_handle; }

private:
    BbxGraph* m_handle { nullptr };
};

} // namespace bbx
```
