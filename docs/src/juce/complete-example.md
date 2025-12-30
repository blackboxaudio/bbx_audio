# Complete Example Walkthrough

A complete example of integrating Rust DSP with a JUCE plugin.

## Project Structure

```
my-utility-plugin/
├── CMakeLists.txt
├── dsp/
│   ├── Cargo.toml
│   ├── parameters.json
│   ├── include/
│   │   ├── bbx_ffi.h
│   │   └── bbx_graph.h
│   └── src/
│       └── lib.rs
├── src/
│   ├── PluginProcessor.cpp
│   ├── PluginProcessor.h
│   ├── PluginEditor.cpp
│   └── PluginEditor.h
└── vendor/
    └── corrosion/
```

## Step 1: Define Parameters

`dsp/parameters.json`:

```json
{
  "parameters": [
    {
      "id": "GAIN",
      "name": "Gain",
      "type": "float",
      "min": -60.0,
      "max": 30.0,
      "defaultValue": 0.0,
      "unit": "dB"
    },
    {
      "id": "PAN",
      "name": "Pan",
      "type": "float",
      "min": -100.0,
      "max": 100.0,
      "defaultValue": 0.0
    },
    {
      "id": "MONO",
      "name": "Mono",
      "type": "boolean",
      "defaultValue": false
    }
  ]
}
```

## Step 2: Rust DSP Implementation

`dsp/Cargo.toml`:

```toml
[package]
name = "dsp"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["staticlib"]

[dependencies]
bbx_plugin = "0.1"
```

`dsp/src/lib.rs`:

```rust
use bbx_plugin::{
    PluginDsp, DspContext, bbx_plugin_ffi,
    blocks::{GainBlock, PannerBlock},
};

// Parameter indices
const PARAM_GAIN: usize = 0;
const PARAM_PAN: usize = 1;
const PARAM_MONO: usize = 2;

pub struct PluginGraph {
    gain: GainBlock<f32>,
    panner: PannerBlock<f32>,
    mono: bool,
}

impl Default for PluginGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginDsp for PluginGraph {
    fn new() -> Self {
        Self {
            gain: GainBlock::new(0.0),
            panner: PannerBlock::new(0.0),
            mono: false,
        }
    }

    fn prepare(&mut self, _context: &DspContext) {
        // Nothing to prepare for these simple blocks
    }

    fn reset(&mut self) {
        // Nothing to reset
    }

    fn apply_parameters(&mut self, params: &[f32]) {
        self.gain.level_db = params[PARAM_GAIN];
        self.panner.position = params[PARAM_PAN];
        self.mono = params[PARAM_MONO] > 0.5;
    }

    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        context: &DspContext,
    ) {
        let num_samples = context.buffer_size;
        let multiplier = self.gain.multiplier();

        // Apply gain
        for ch in 0..inputs.len().min(outputs.len()) {
            for i in 0..num_samples {
                outputs[ch][i] = inputs[ch][i] * multiplier;
            }
        }

        // Apply mono summing if enabled
        if self.mono && outputs.len() >= 2 {
            for i in 0..num_samples {
                let sum = (outputs[0][i] + outputs[1][i]) * 0.5;
                outputs[0][i] = sum;
                outputs[1][i] = sum;
            }
        }

        // Apply panning
        if outputs.len() >= 2 {
            let (left_gain, right_gain) = self.panner.gains();
            for i in 0..num_samples {
                outputs[0][i] *= left_gain;
                outputs[1][i] *= right_gain;
            }
        }
    }
}

bbx_plugin_ffi!(PluginGraph);
```

## Step 3: CMake Configuration

`CMakeLists.txt`:

```cmake
cmake_minimum_required(VERSION 3.15)
project(MyUtilityPlugin VERSION 1.0.0)

add_subdirectory(JUCE)
add_subdirectory(vendor/corrosion)

corrosion_import_crate(MANIFEST_PATH dsp/Cargo.toml)

juce_add_plugin(MyUtilityPlugin
    PLUGIN_MANUFACTURER_CODE Bbxa
    PLUGIN_CODE Util
    FORMATS AU VST3 Standalone
    PRODUCT_NAME "My Utility Plugin")

target_sources(MyUtilityPlugin PRIVATE
    src/PluginProcessor.cpp
    src/PluginEditor.cpp)

target_link_libraries(MyUtilityPlugin PRIVATE
    juce::juce_audio_processors
    juce::juce_audio_utils
    dsp)

target_include_directories(MyUtilityPlugin PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/dsp/include)
```

## Step 4: JUCE Processor

`src/PluginProcessor.h`:

```cpp
#pragma once

#include <juce_audio_processors/juce_audio_processors.h>
#include <bbx_graph.h>
#include <bbx_ffi.h>
#include <array>
#include <vector>

class PluginAudioProcessor : public juce::AudioProcessor {
public:
    PluginAudioProcessor();
    ~PluginAudioProcessor() override = default;

    void prepareToPlay(double sampleRate, int samplesPerBlock) override;
    void releaseResources() override;
    void processBlock(juce::AudioBuffer<float>&, juce::MidiBuffer&) override;

    juce::AudioProcessorEditor* createEditor() override;
    bool hasEditor() const override { return true; }

    const juce::String getName() const override { return "My Utility Plugin"; }
    bool acceptsMidi() const override { return false; }
    bool producesMidi() const override { return false; }
    double getTailLengthSeconds() const override { return 0.0; }

    int getNumPrograms() override { return 1; }
    int getCurrentProgram() override { return 0; }
    void setCurrentProgram(int) override {}
    const juce::String getProgramName(int) override { return {}; }
    void changeProgramName(int, const juce::String&) override {}

    void getStateInformation(juce::MemoryBlock& destData) override;
    void setStateInformation(const void* data, int sizeInBytes) override;

    juce::AudioProcessorValueTreeState parameters;

private:
    juce::AudioProcessorValueTreeState::ParameterLayout createParameterLayout();

    bbx::Graph m_rustDsp;
    std::vector<float> m_paramBuffer;

    std::atomic<float>* gainParam = nullptr;
    std::atomic<float>* panParam = nullptr;
    std::atomic<float>* monoParam = nullptr;

    static constexpr size_t MAX_CHANNELS = 8;
    std::array<const float*, MAX_CHANNELS> m_inputPtrs {};
    std::array<float*, MAX_CHANNELS> m_outputPtrs {};
};
```

`src/PluginProcessor.cpp`:

```cpp
#include "PluginProcessor.h"
#include "PluginEditor.h"

PluginAudioProcessor::PluginAudioProcessor()
    : AudioProcessor(BusesProperties()
        .withInput("Input", juce::AudioChannelSet::stereo(), true)
        .withOutput("Output", juce::AudioChannelSet::stereo(), true))
    , parameters(*this, nullptr, "Parameters", createParameterLayout())
{
    gainParam = parameters.getRawParameterValue("gain");
    panParam = parameters.getRawParameterValue("pan");
    monoParam = parameters.getRawParameterValue("mono");

    m_paramBuffer.resize(PARAM_COUNT);
}

juce::AudioProcessorValueTreeState::ParameterLayout
PluginAudioProcessor::createParameterLayout()
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

void PluginAudioProcessor::prepareToPlay(double sampleRate, int samplesPerBlock)
{
    m_rustDsp.Prepare(
        sampleRate,
        static_cast<uint32_t>(samplesPerBlock),
        static_cast<uint32_t>(getTotalNumOutputChannels()));
}

void PluginAudioProcessor::releaseResources()
{
    m_rustDsp.Reset();
}

void PluginAudioProcessor::processBlock(juce::AudioBuffer<float>& buffer,
                                         juce::MidiBuffer&)
{
    juce::ScopedNoDenormals noDenormals;

    auto numChannels = static_cast<uint32_t>(buffer.getNumChannels());
    auto numSamples = static_cast<uint32_t>(buffer.getNumSamples());
    numChannels = std::min(numChannels, static_cast<uint32_t>(MAX_CHANNELS));

    m_paramBuffer[PARAM_GAIN] = gainParam->load();
    m_paramBuffer[PARAM_PAN] = panParam->load();
    m_paramBuffer[PARAM_MONO] = monoParam->load();

    for (uint32_t ch = 0; ch < numChannels; ++ch) {
        m_inputPtrs[ch] = buffer.getReadPointer(static_cast<int>(ch));
        m_outputPtrs[ch] = buffer.getWritePointer(static_cast<int>(ch));
    }

    m_rustDsp.Process(
        m_inputPtrs.data(),
        m_outputPtrs.data(),
        numChannels,
        numSamples,
        m_paramBuffer.data(),
        static_cast<uint32_t>(m_paramBuffer.size()));
}

juce::AudioProcessorEditor* PluginAudioProcessor::createEditor()
{
    return new PluginEditor(*this);
}

void PluginAudioProcessor::getStateInformation(juce::MemoryBlock& destData)
{
    auto state = parameters.copyState();
    std::unique_ptr<juce::XmlElement> xml(state.createXml());
    copyXmlToBinary(*xml, destData);
}

void PluginAudioProcessor::setStateInformation(const void* data, int sizeInBytes)
{
    std::unique_ptr<juce::XmlElement> xmlState(getXmlFromBinary(data, sizeInBytes));
    if (xmlState && xmlState->hasTagName(parameters.state.getType()))
        parameters.replaceState(juce::ValueTree::fromXml(*xmlState));
}

juce::AudioProcessor* JUCE_CALLTYPE createPluginFilter()
{
    return new PluginAudioProcessor();
}
```

## Step 5: Build

```bash
# Configure
cmake -B build -DCMAKE_BUILD_TYPE=Release

# Build
cmake --build build --config Release
```

The plugin will be in `build/MyUtilityPlugin_artefacts/`.
