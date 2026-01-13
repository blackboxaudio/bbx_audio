# Head-Related Transfer Functions (HRTF)

Head-Related Transfer Functions model how sounds arriving from different directions are filtered by the listener's head, pinnae (outer ears), and torso before reaching the eardrums. This filtering enables humans to localize sounds in 3D space using only two ears.

## What is HRTF?

When a sound wave travels from a source to a listener, it arrives at each ear with different characteristics depending on the source's location:

- **Interaural Time Difference (ITD)**: Sound arrives at the near ear before the far ear
- **Interaural Level Difference (ILD)**: The head shadows high frequencies, making the far ear quieter
- **Spectral Cues**: The pinnae create frequency-dependent reflections and diffractions that encode elevation and front/back information

### Spatial Coordinate System

HRTF measurements use a spherical coordinate system centered on the listener:

- **Azimuth (θ)**: The horizontal angle around the listener, measured in degrees. 0° is directly in front, 90° is to the right, 180° (or -180°) is directly behind, and -90° is to the left.
- **Elevation (φ)**: The vertical angle above or below the horizontal plane. 0° is ear-level, +90° is directly above, and -90° is directly below.
- **Frequency (ω)**: The angular frequency of the sound wave in radians per second (ω = 2πf where f is frequency in Hz). HRTFs describe how each frequency component is modified differently based on direction.

An HRTF captures all these cues as a frequency-domain transfer function $H(\omega, \theta, \phi)$.

## HRIR: Time-Domain Representation

The **Head-Related Impulse Response (HRIR)** is the time-domain equivalent of an HRTF. It represents what happens to an impulse (click) traveling from a specific direction:

$$
\text{HRIR}(t, \theta, \phi) = \mathcal{F}^{-1}\left\{ \text{HRTF}(\omega, \theta, \phi) \right\}
$$

HRIRs are typically 128-512 samples long (2.7-10.7ms at 48kHz) and encode the full binaural transformation for a single direction.

## Mathematical Foundation

### Binaural Rendering via Convolution

To render a mono source $x(t)$ at position $(\theta, \phi)$, convolve it with the appropriate HRIRs:

$$
\begin{aligned}
y_L(t) &= x(t) * h_L(t, \theta, \phi) \\
y_R(t) &= x(t) * h_R(t, \theta, \phi)
\end{aligned}
$$

where $*$ denotes convolution and $h_L$, $h_R$ are the left and right ear HRIRs.

### Time-Domain Convolution

For an HRIR of length $N$ and input signal $x[n]$, the output $y[n]$ at sample $n$ is:

$$
y[n] = \sum_{k=0}^{N-1} x[n-k] \cdot h[k]
$$

This is an FIR filter operation with the HRIR as coefficients.

### Complexity Analysis

For each sample:
- **Multiplications**: $N$ (HRIR length)
- **Additions**: $N-1$

Total per audio frame of $B$ samples:
$$
\text{Operations} = B \times N \times 2 \quad \text{(left + right ears)}
$$

### Spherical Harmonics Decomposition

For ambisonic signals, we decode to virtual speaker positions then apply HRTFs. Each virtual speaker's signal is computed by weighting ambisonic channels with spherical harmonic coefficients:

$$
s_i = \sum_{l=0}^{L} \sum_{m=-l}^{l} Y_l^m(\theta_i, \phi_i) \cdot a_{l,m}
$$

where:
- $Y_l^m(\theta, \phi)$ are real spherical harmonics (SN3D normalized)
- $a_{l,m}$ is the ambisonic channel for order $l$, degree $m$
- $(\theta_i, \phi_i)$ is the virtual speaker's position

## Implementation in bbx_audio

### Virtual Speaker Approach

`BinauralDecoderBlock` uses a virtual speaker array for HRTF rendering:

1. **Decode** ambisonic input to $N$ virtual speaker signals using SH coefficients
2. **Convolve** each speaker signal with position-specific HRIRs
3. **Sum** all convolved outputs for left and right ears

$$
\begin{aligned}
y_L &= \sum_{i=0}^{N-1} s_i * h_{L,i} \\
y_R &= \sum_{i=0}^{N-1} s_i * h_{R,i}
\end{aligned}
$$

### HRIR Data

The implementation uses HRIR measurements from the MIT KEMAR database:

- **Source**: MIT Media Lab KEMAR HRTF Database (Gardner & Martin, 1994)
- **Mannequin**: KEMAR (Knowles Electronics Manikin for Acoustic Research)
- **Length**: 256 samples per HRIR
- **Positions**: Quantized to cardinal directions (front, back, left, right, and 45° diagonals)

### Spherical Harmonic Coefficients

For a virtual speaker at azimuth $\theta$ and elevation $\phi$, the real SH coefficients (ACN/SN3D) are:

**Order 0:**
$$
Y_0^0 = 1
$$

**Order 1:**
$$
\begin{aligned}
Y_1^{-1} &= \cos\phi \cdot \sin\theta \quad \text{(Y channel)} \\
Y_1^0 &= \sin\phi \quad \text{(Z channel)} \\
Y_1^1 &= \cos\phi \cdot \cos\theta \quad \text{(X channel)}
\end{aligned}
$$

**Order 2:**
$$
\begin{aligned}
Y_2^{-2} &= \sqrt{\frac{3}{4}} \cos^2\phi \cdot \sin(2\theta) \quad \text{(V channel)} \\
Y_2^{-1} &= \sqrt{\frac{3}{4}} \sin(2\phi) \cdot \sin\theta \quad \text{(T channel)} \\
Y_2^0 &= \frac{3\sin^2\phi - 1}{2} \quad \text{(R channel)} \\
Y_2^1 &= \sqrt{\frac{3}{4}} \sin(2\phi) \cdot \cos\theta \quad \text{(S channel)} \\
Y_2^2 &= \sqrt{\frac{3}{4}} \cos^2\phi \cdot \cos(2\theta) \quad \text{(U channel)}
\end{aligned}
$$

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

This achieves $O(N)$ convolution per sample where $N$ is HRIR length.

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

4 virtual speakers at $\pm 45°$ and $\pm 135°$ azimuth:

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
| Channel | Azimuth |
|---------|---------|
| L/R | $\pm 30°$ |
| C | $0°$ |
| LFE | $0°$ (non-directional) |
| Ls/Rs | $\pm 110°$ |

**7.1 (ITU-R BS.2051):**
| Channel | Azimuth |
|---------|---------|
| L/R | $\pm 30°$ |
| C | $0°$ |
| LFE | $0°$ |
| Ls/Rs | $\pm 90°$ (side) |
| Lrs/Rrs | $\pm 150°$ (rear) |

## Performance Considerations

### CPU Cost

HRTF convolution complexity per audio frame:

$$
\text{Operations} = B \times N_{speakers} \times L_{HRIR} \times 2
$$

For a 512-sample buffer with 4-speaker FOA and 256-sample HRIRs:
$$
512 \times 4 \times 256 \times 2 = 1,048,576 \text{ multiply-adds}
$$

### Memory Usage

- **HRIR storage**: $N_{speakers} \times 2 \times L_{HRIR} \times \text{sizeof}(f32)$
- **Signal buffers**: $N_{speakers} \times L_{HRIR} \times \text{sizeof}(f32)$

For 4-speaker FOA with 256-sample HRIRs:
- HRIRs: $4 \times 2 \times 256 \times 4 = 8$ KB
- Buffers: $4 \times 256 \times 4 = 4$ KB

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

- Blauert, J. (1997). *Spatial Hearing: The Psychophysics of Human Sound Localization*. MIT Press.
- Zotter, F. & Frank, M. (2019). *Ambisonics: A Practical 3D Audio Theory*. Springer.
- Wightman, F.L. & Kistler, D.J. (1989). "Headphone simulation of free-field listening." *JASA*, 85(2), 858-867.
- MIT KEMAR Database: [https://sound.media.mit.edu/resources/KEMAR.html](https://sound.media.mit.edu/resources/KEMAR.html)
- AES69-2015: *AES standard for file exchange - Spatial acoustic data file format (SOFA)*
