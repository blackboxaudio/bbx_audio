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
#include <array>
#include <vector>

class PluginAudioProcessor : public juce::AudioProcessor {
public:
    PluginAudioProcessor();
    ~PluginAudioProcessor() override;

    void prepareToPlay(double sampleRate, int samplesPerBlock) override;
    void releaseResources() override;
    void processBlock(juce::AudioBuffer<float>&, juce::MidiBuffer&) override;

    // ... other AudioProcessor methods

private:
    bbx::Graph m_rustDsp;
    std::vector<float> m_paramBuffer;

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
{
    // Allocate parameter buffer
    m_paramBuffer.resize(PARAM_COUNT);
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
void PluginAudioProcessor::processBlock(juce::AudioBuffer<float>& buffer,
                                         juce::MidiBuffer& midiMessages)
{
    juce::ScopedNoDenormals noDenormals;

    auto numChannels = static_cast<uint32_t>(buffer.getNumChannels());
    auto numSamples = static_cast<uint32_t>(buffer.getNumSamples());

    // Clamp to max supported channels
    numChannels = std::min(numChannels, static_cast<uint32_t>(MAX_CHANNELS));

    // Gather current parameter values
    m_paramBuffer[PARAM_GAIN] = *gainParameter;
    m_paramBuffer[PARAM_PAN] = *panParameter;
    m_paramBuffer[PARAM_MONO] = *monoParameter ? 1.0f : 0.0f;
    // ... more parameters

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

### With APVTS

Using `AudioProcessorValueTreeState`:

```cpp
class PluginAudioProcessor : public juce::AudioProcessor {
private:
    juce::AudioProcessorValueTreeState parameters;

    std::atomic<float>* gainParam = nullptr;
    std::atomic<float>* panParam = nullptr;
    // ...
};

PluginAudioProcessor::PluginAudioProcessor()
    : parameters(*this, nullptr, "Parameters", createParameterLayout())
{
    gainParam = parameters.getRawParameterValue("gain");
    panParam = parameters.getRawParameterValue("pan");
}

void PluginAudioProcessor::processBlock(...)
{
    m_paramBuffer[PARAM_GAIN] = gainParam->load();
    m_paramBuffer[PARAM_PAN] = panParam->load();
    // ...
}
```

### Parameter Layout

Create parameters matching your Rust indices:

```cpp
juce::AudioProcessorValueTreeState::ParameterLayout createParameterLayout()
{
    std::vector<std::unique_ptr<juce::RangedAudioParameter>> params;

    params.push_back(std::make_unique<juce::AudioParameterFloat>(
        "gain", "Gain",
        juce::NormalisableRange<float>(-60.0f, 30.0f, 0.1f),
        0.0f));

    params.push_back(std::make_unique<juce::AudioParameterFloat>(
        "pan", "Pan",
        juce::NormalisableRange<float>(-100.0f, 100.0f, 1.0f),
        0.0f));

    params.push_back(std::make_unique<juce::AudioParameterBool>(
        "mono", "Mono", false));

    return { params.begin(), params.end() };
}
```

## Thread Safety Notes

- `processBlock()` runs on the audio thread
- Parameter reads should use atomics
- Never allocate memory in `processBlock()`
- The `bbx::Graph` is already designed for audio thread use
