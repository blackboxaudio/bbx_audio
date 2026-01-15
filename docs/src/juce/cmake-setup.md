# CMake with Corrosion

Configure CMake to build and link your Rust DSP crate.

## CMakeLists.txt

Add these sections to your plugin's `CMakeLists.txt`:

```cmake
# Add Corrosion for Rust integration
add_subdirectory(vendor/corrosion)

# Import the Rust crate
corrosion_import_crate(MANIFEST_PATH dsp/Cargo.toml)

# Your plugin target (created by JUCE's juce_add_plugin)
# Replace ${PLUGIN_TARGET} with your actual target name

# Include the FFI headers
target_include_directories(${PLUGIN_TARGET} PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/dsp/include)

# Link the Rust library
target_link_libraries(${PLUGIN_TARGET} PRIVATE dsp)
```

## Full Example

A complete `CMakeLists.txt` example:

```cmake
cmake_minimum_required(VERSION 3.15)
project(MyPlugin VERSION 1.0.0)

# Add JUCE (adjust path as needed)
add_subdirectory(JUCE)

# Add Corrosion
add_subdirectory(vendor/corrosion)

# Import Rust crate
corrosion_import_crate(MANIFEST_PATH dsp/Cargo.toml)

# Define the plugin
juce_add_plugin(MyPlugin
    PLUGIN_MANUFACTURER_CODE Mfr1
    PLUGIN_CODE Plg1
    FORMATS AU VST3 Standalone
    PRODUCT_NAME "My Plugin")

# Add JUCE modules
target_link_libraries(MyPlugin PRIVATE
    juce::juce_audio_processors
    juce::juce_audio_utils)

# Include FFI headers
target_include_directories(MyPlugin PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/dsp/include)

# Link Rust library
target_link_libraries(MyPlugin PRIVATE dsp)
```

## Platform-Specific Notes

### macOS

No additional configuration needed. Corrosion handles universal binary creation.

### Windows

Ensure Rust is in your PATH. You may need to specify the MSVC toolchain:

```bash
rustup default stable-msvc
```

### Linux

Install required development packages:

```bash
sudo apt install libasound2-dev
```

## Build Commands

```bash
# Configure
cmake -B build -DCMAKE_BUILD_TYPE=Release

# Build
cmake --build build --config Release
```

## Verify the Build

After building, verify the Rust library was created:

```bash
# macOS/Linux - check for the static library
ls build/cargo/build/*/release/libdsp.a

# Windows - check for the static library
dir build\cargo\build\*\release\dsp.lib
```

If the library is missing, check the `build/cargo/` directory for Rust build errors. Common issues:
- Missing Rust toolchain (run `rustup show`)
- Cargo.toml syntax errors
- Missing `staticlib` crate type

## Troubleshooting

### "Cannot find -ldsp"

Ensure Corrosion successfully built the Rust crate. Check `build/cargo/` for build artifacts.

### Linking Errors

Verify the crate type in `Cargo.toml` is set to `staticlib`:

```toml
[lib]
crate-type = ["staticlib"]
```

### Header Not Found

Check that `target_include_directories` points to the correct path containing `bbx_ffi.h`.
