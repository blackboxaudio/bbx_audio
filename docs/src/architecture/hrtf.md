# Head-Related Transfer Functions (HRTF)

Head-Related Transfer Functions model how sounds arriving from different directions are filtered by the listener's head, pinnae (outer ears), and torso before reaching the eardrums. This filtering enables humans to localize sounds in 3D space using only two ears.

## What is HRTF?

When a sound wave travels from a source to a listener, it arrives at each ear with different characteristics depending on the source's location:

- **Interaural Time Difference (ITD)**: Sound arrives at the near ear before the far ear
- **Interaural Level Difference (ILD)**: The head shadows high frequencies, making the far ear quieter
- **Spectral Cues**: The pinnae create frequency-dependent reflections and diffractions that encode elevation and front/back information

An HRTF captures all these cues as a frequency-domain transfer function H(ω, θ, φ) where ω is frequency, θ is azimuth, and φ is elevation.

## HRIR: Time-Domain Representation

The **Head-Related Impulse Response (HRIR)** is the time-domain equivalent of an HRTF. It represents what happens to an impulse (click) traveling from a specific direction:

```
HRIR(t, θ, φ) = IFFT{ HRTF(ω, θ, φ) }
```

HRIRs are typically 128-512 samples long (2.7-10.7ms at 48kHz) and encode the full binaural transformation for a single direction.

## Mathematical Foundation

### Binaural Rendering via Convolution

To render a mono source at position (θ, φ), convolve it with the appropriate HRIRs:

```
left_ear(t)  = source(t) * hrir_left(t, θ, φ)
right_ear(t) = source(t) * hrir_right(t, θ, φ)
```

Where `*` denotes convolution.

### Time-Domain Convolution

For an HRIR of length N and input signal x[n], the output y[n] at sample n is:

```
y[n] = Σ(k=0 to N-1) x[n-k] · h[k]
```

This is an FIR filter operation with the HRIR as coefficients.

### Spherical Harmonics Decomposition

For ambisonic signals, we decode to virtual speaker positions then apply HRTFs. Each virtual speaker's signal is computed by weighting ambisonic channels with spherical harmonic coefficients:

```
speaker_signal = Σ(l,m) Y_l^m(θ, φ) · ambisonic_channel[acn(l,m)]
```

Where:
- `Y_l^m(θ, φ)` are real spherical harmonics (SN3D normalized)
- `acn(l,m)` maps degree l and order m to ACN channel index
- (θ, φ) is the virtual speaker's position

## Implementation in bbx_audio

### Virtual Speaker Approach

`BinauralDecoderBlock` uses a virtual speaker array for HRTF rendering:

1. **Decode** ambisonic input to N virtual speaker signals using SH coefficients
2. **Convolve** each speaker signal with position-specific HRIRs
3. **Sum** all convolved outputs for left and right ears

```
left_ear  = Σ(i=0 to N-1) decode(ambi, speaker[i]) * hrir_left[i]
right_ear = Σ(i=0 to N-1) decode(ambi, speaker[i]) * hrir_right[i]
```

### HRIR Data

The implementation uses HRIR measurements from the MIT KEMAR database:

- **Source**: MIT Media Lab KEMAR HRTF Database (Gardner & Martin, 1994)
- **Mannequin**: KEMAR (Knowles Electronics Manikin for Acoustic Research)
- **Length**: 256 samples per HRIR
- **Positions**: Quantized to cardinal directions (front, back, left, right, and 45° diagonals)

### Spherical Harmonic Coefficients

For a virtual speaker at azimuth θ and elevation φ, the real SH coefficients (ACN/SN3D) are:

**Order 0:**
```
Y_0^0 = 1
```

**Order 1:**
```
Y_1^-1 = cos(φ) · sin(θ)     [Y channel]
Y_1^0  = sin(φ)              [Z channel]
Y_1^1  = cos(φ) · cos(θ)     [X channel]
```

**Order 2:**
```
Y_2^-2 = √(3/4) · cos²(φ) · sin(2θ)           [V channel]
Y_2^-1 = √(3/4) · sin(2φ) · sin(θ)            [T channel]
Y_2^0  = (3sin²(φ) - 1) / 2                   [R channel]
Y_2^1  = √(3/4) · sin(2φ) · cos(θ)            [S channel]
Y_2^2  = √(3/4) · cos²(φ) · cos(2θ)           [U channel]
```

### Circular Buffer Convolution

For efficient realtime processing, convolution uses a circular buffer:

```rust
// Store incoming sample
buffer[write_pos] = input_sample;

// Convolve with HRIR
for k in 0..hrir_length {
    let buf_idx = (write_pos + hrir_length - k) % hrir_length;
    output += buffer[buf_idx] * hrir[k];
}

// Advance write position
write_pos = (write_pos + 1) % hrir_length;
```

This achieves O(N) convolution per sample where N is HRIR length.

## Decoding Strategies

`BinauralDecoderBlock` offers two strategies:

### Matrix Strategy (Lightweight)

Uses pre-computed ILD-based coefficients without HRTF convolution:
- Lower CPU usage
- Basic left/right separation
- No ITD or spectral cues
- Sounds may appear "inside the head"

### HRTF Strategy (Default)

Full HRTF convolution with virtual speakers:
- Higher CPU usage (proportional to HRIR length × speaker count)
- Accurate ITD, ILD, and spectral cues
- Better externalization (sounds appear outside the head)
- More convincing 3D positioning

## Virtual Speaker Layouts

### Ambisonic Decoding (FOA)

4 virtual speakers at ±45° and ±135° azimuth:

```
        Front
          |
   FL ----+---- FR    (±45°)
          |
          |
   RL ----+---- RR    (±135°)
          |
        Rear
```

### Surround Sound (5.1/7.1)

Standard ITU-R speaker positions:

**5.1 (ITU-R BS.775-1):**
- L/R: ±30°
- C: 0°
- LFE: 0° (non-directional)
- Ls/Rs: ±110°

**7.1 (ITU-R BS.2051):**
- L/R: ±30°
- C: 0°
- LFE: 0°
- Ls/Rs: ±90° (side)
- Lrs/Rrs: ±150° (rear)

## Performance Considerations

### CPU Cost

HRTF convolution complexity per audio frame:
```
Operations = samples × speakers × hrir_length
           = 512 × 4 × 256 = 524,288 multiply-adds (FOA)
```

### Memory Usage

- HRIR storage: `speakers × 2 × hrir_length × sizeof(f32)`
- Signal buffers: `speakers × hrir_length × sizeof(f32)`

For 4-speaker FOA with 256-sample HRIRs:
- HRIRs: 4 × 2 × 256 × 4 = 8 KB
- Buffers: 4 × 256 × 4 = 4 KB

### Realtime Safety

The implementation is fully realtime-safe:
- All buffers pre-allocated at construction
- No allocations during `process()`
- No locks or system calls
- Circular buffer avoids memory copies

## Limitations

### HRIR Resolution

The current implementation uses a limited set of HRIR positions. Sounds between measured positions may exhibit less precise localization compared to interpolated or individualized HRTFs.

### Head Tracking

Without head tracking, the virtual sound stage rotates with the listener's head. For immersive applications, consider integrating gyroscope data to counter-rotate the soundfield.

### Individualization

Generic HRTFs (like KEMAR) work reasonably well for most listeners but optimal spatial accuracy requires individually-measured HRTFs that account for each person's unique ear geometry.

## Further Reading

- Blauert, J. (1997). *Spatial Hearing: The Psychophysics of Human Sound Localization*
- Zotter, F. & Frank, M. (2019). *Ambisonics: A Practical 3D Audio Theory*
- MIT KEMAR Database: <https://sound.media.mit.edu/resources/KEMAR.html>
- AES69-2015: *AES standard for file exchange - Spatial acoustic data file format (SOFA)*
