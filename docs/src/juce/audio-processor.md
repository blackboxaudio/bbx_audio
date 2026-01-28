# AudioProcessor Integration

Integrate `bbx::Graph` with your JUCE AudioProcessor.

## Overview

The integration pattern:

1. Store `bbx::Graph` as a member
2. Call `Prepare()` in `prepareToPlay()`
3. Call `Reset()` in `releaseResources()`
4. Call `Process()` in `processBlock()`

## Processor Header

```cpp
#pragma once

#include <juce_audio_processors/juce_audio_processors.h>
#include <bbx_graph.h>
#include <bbx_params.h>
#include <array>
#include <atomic>
#include <vector>

class PluginAudioProcessor : public juce::AudioProcessor {
public:
    PluginAudioProcessor();
    ~PluginAudioProcessor() override;

    void prepareToPlay(double sampleRate, int samplesPerBlock) override;
    void releaseResources() override;
    void processBlock(juce::SampleBuffer<float>&, juce::MidiBuffer&) override;

    // ... other AudioProcessor methods

private:
    juce::AudioProcessorValueTreeState m_parameters;

    bbx::Graph m_rustDsp;
    std::vector<float> m_paramBuffer;
    std::array<std::atomic<float>*, PARAM_COUNT> m_paramPointers {};

    // Pointer arrays for FFI
    static constexpr size_t MAX_CHANNELS = 8;
    std::array<const float*, MAX_CHANNELS> m_inputPtrs {};
    std::array<float*, MAX_CHANNELS> m_outputPtrs {};
};
```

## Implementation

### Constructor

```cpp
PluginAudioProcessor::PluginAudioProcessor()
    : AudioProcessor(/* bus layout */)
    , m_parameters(*this, nullptr, "Parameters", createParameterLayout())
{
    // Allocate parameter buffer
    m_paramBuffer.resize(PARAM_COUNT);

    // Cache parameter pointers for efficient access in processBlock
    for (size_t i = 0; i < PARAM_COUNT; ++i) {
        m_paramPointers[i] = m_parameters.getRawParameterValue(juce::String(PARAM_IDS[i]));
    }
}
```

### prepareToPlay

```cpp
void PluginAudioProcessor::prepareToPlay(double sampleRate, int samplesPerBlock)
{
    BbxError err = m_rustDsp.Prepare(
        sampleRate,
        static_cast<uint32_t>(samplesPerBlock),
        static_cast<uint32_t>(getTotalNumOutputChannels())
    );

    if (err != BBX_ERROR_OK) {
        DBG("Failed to prepare Rust DSP: " + juce::String(static_cast<int>(err)));
    }
}
```

### releaseResources

```cpp
void PluginAudioProcessor::releaseResources()
{
    m_rustDsp.Reset();
}
```

### processBlock

```cpp
void PluginAudioProcessor::processBlock(juce::SampleBuffer<float>& buffer,
                                         juce::MidiBuffer& midiMessages)
{
    juce::ScopedNoDenormals noDenormals;

    auto numChannels = static_cast<uint32_t>(buffer.getNumChannels());
    auto numSamples = static_cast<uint32_t>(buffer.getNumSamples());

    // Clamp to max supported channels
    numChannels = std::min(numChannels, static_cast<uint32_t>(MAX_CHANNELS));

    // Load parameter values from cached atomic pointers
    for (size_t i = 0; i < PARAM_COUNT; ++i) {
        m_paramBuffer[i] = m_paramPointers[i] ? m_paramPointers[i]->load() : 0.0f;
    }

    // Build pointer arrays
    for (uint32_t ch = 0; ch < numChannels; ++ch) {
        m_inputPtrs[ch] = buffer.getReadPointer(static_cast<int>(ch));
        m_outputPtrs[ch] = buffer.getWritePointer(static_cast<int>(ch));
    }

    // Process through Rust DSP
    m_rustDsp.Process(
        m_inputPtrs.data(),
        m_outputPtrs.data(),
        numChannels,
        numSamples,
        m_paramBuffer.data(),
        static_cast<uint32_t>(m_paramBuffer.size())
    );
}
```

## Parameter Integration

The recommended approach uses `PARAM_IDS` from the generated header for dynamic iteration:

```cpp
// In constructor - cache all parameter pointers
for (size_t i = 0; i < PARAM_COUNT; ++i) {
    m_paramPointers[i] = m_parameters.getRawParameterValue(juce::String(PARAM_IDS[i]));
}

// In processBlock - load all values dynamically
for (size_t i = 0; i < PARAM_COUNT; ++i) {
    m_paramBuffer[i] = m_paramPointers[i] ? m_paramPointers[i]->load() : 0.0f;
}
```

This eliminates per-parameter boilerplate. When adding new parameters, only update `parameters.json` and the Rust `apply_parameters()` method.

### Parameter Layout

Create the layout from JSON using `cortex::ParameterManager` or manually:

```cpp
juce::AudioProcessorValueTreeState::ParameterLayout createParameterLayout()
{
    // Option 1: Load from embedded JSON (recommended)
    juce::String json = juce::String::createStringFromData(
        PluginData::parameters_json, PluginData::parameters_jsonSize);
    auto params = cortex::ParameterManager::LoadParametersFromJson(json);
    return cortex::ParameterManager::CreateParameterLayout(params);

    // Option 2: Manual definition
    std::vector<std::unique_ptr<juce::RangedAudioParameter>> params;
    params.push_back(std::make_unique<juce::AudioParameterFloat>(
        "GAIN", "Gain",
        juce::NormalisableRange<float>(-60.0f, 30.0f, 0.1f),
        0.0f));
    // ... more parameters
    return { params.begin(), params.end() };
}
```

## Thread Safety Notes

- `processBlock()` runs on the audio thread
- Parameter reads should use atomics
- Never allocate memory in `processBlock()`
- The `bbx::Graph` is already designed for audio thread use

## MIDI Integration

For synthesizers that need MIDI input, convert JUCE's `MidiBuffer` to `BbxMidiEvent` array:

```cpp
// In processor header
static constexpr size_t MAX_MIDI_EVENTS = 128;
std::array<BbxMidiEvent, MAX_MIDI_EVENTS> m_midiEvents {};

// Helper function to convert JUCE MidiBuffer to BbxMidiEvent array
uint32_t convertMidiBuffer(const juce::MidiBuffer& buffer,
                           BbxMidiEvent* events,
                           uint32_t maxEvents)
{
    uint32_t count = 0;
    for (const auto metadata : buffer) {
        if (count >= maxEvents) break;

        const auto msg = metadata.getMessage();
        auto& event = events[count];

        event.sample_offset = static_cast<uint32_t>(metadata.samplePosition);
        event.message.channel = static_cast<uint8_t>(msg.getChannel() - 1);

        if (msg.isNoteOn()) {
            event.message.status = BBX_MIDI_STATUS_NOTE_ON;
            event.message.data_1 = static_cast<uint8_t>(msg.getNoteNumber());
            event.message.data_2 = static_cast<uint8_t>(msg.getVelocity());
        } else if (msg.isNoteOff()) {
            event.message.status = BBX_MIDI_STATUS_NOTE_OFF;
            event.message.data_1 = static_cast<uint8_t>(msg.getNoteNumber());
            event.message.data_2 = 0;
        } else if (msg.isController()) {
            event.message.status = BBX_MIDI_STATUS_CONTROL_CHANGE;
            event.message.data_1 = static_cast<uint8_t>(msg.getControllerNumber());
            event.message.data_2 = static_cast<uint8_t>(msg.getControllerValue());
        } else if (msg.isPitchWheel()) {
            event.message.status = BBX_MIDI_STATUS_PITCH_WHEEL;
            int pitchValue = msg.getPitchWheelValue() - 8192;
            event.message.data_1 = static_cast<uint8_t>(pitchValue & 0x7F);
            event.message.data_2 = static_cast<uint8_t>((pitchValue >> 7) & 0x7F);
        } else {
            continue;
        }

        count++;
    }
    return count;
}
```

Use in `processBlock()`:

```cpp
void PluginAudioProcessor::processBlock(juce::SampleBuffer<float>& buffer,
                                         juce::MidiBuffer& midiMessages)
{
    // ... parameter loading ...

    // Convert MIDI for synths
    uint32_t numMidiEvents = convertMidiBuffer(midiMessages, m_midiEvents.data(), MAX_MIDI_EVENTS);

    // Process with MIDI
    m_rustDsp.Process(
        m_inputPtrs.data(),
        m_outputPtrs.data(),
        numChannels,
        numSamples,
        m_paramBuffer.data(),
        static_cast<uint32_t>(m_paramBuffer.size()),
        m_midiEvents.data(),
        numMidiEvents);
}
```

For effect plugins that don't need MIDI, pass `nullptr` and `0` for the MIDI parameters.
