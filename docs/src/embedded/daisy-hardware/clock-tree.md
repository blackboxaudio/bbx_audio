# Clock Tree

Accurate clock configuration is critical for audio. A clock error of just 0.01% causes audible pitch drift and sync issues. This document explains how the STM32H750 generates precise audio clocks.

## Clock Sources

The STM32H750 has several clock sources:

| Source | Frequency | Accuracy | Use |
|--------|-----------|----------|-----|
| HSI | 64 MHz | ±1% | Internal RC, backup |
| HSE | 16 MHz | ±10 ppm | External crystal, primary |
| LSI | 32 kHz | ±5% | Low-power RTC |
| LSE | 32.768 kHz | ±20 ppm | External RTC crystal |

**Daisy uses HSE** (High-Speed External): A 16 MHz crystal oscillator provides the stable reference for all clocks.

## PLL Architecture

PLLs (Phase-Locked Loops) multiply the input frequency to generate higher clocks:

```mermaid
graph LR
    HSE[HSE<br/>16 MHz] --> DIVM[/M<br/>Divider]
    DIVM --> VCO[VCO<br/>Multiplier]
    VCO --> DIVP[/P] --> PLLP[PLL_P output]
    VCO --> DIVQ[/Q] --> PLLQ[PLL_Q output]
    VCO --> DIVR[/R] --> PLLR[PLL_R output]
```

**PLL Formula**:
```
VCO = (HSE / M) × N
PLL_P = VCO / P
PLL_Q = VCO / Q
PLL_R = VCO / R
```

The STM32H750 has three PLLs:
- **PLL1**: System clock (CPU, buses)
- **PLL2**: Peripheral clocks
- **PLL3**: Audio clocks (SAI)

## System Clock Configuration

PLL1 generates the main system clock:

```
HSE = 16 MHz
PLL1_M = 4     → 16 / 4 = 4 MHz (VCO input)
PLL1_N = 400   → 4 × 400 = 1600 MHz (VCO)
PLL1_P = 4     → 1600 / 4 = 400 MHz (System clock)
```

**Bus Clocks**:
```
SYSCLK = 400 MHz
AHB    = 200 MHz (SYSCLK / 2)
APB1   = 100 MHz (AHB / 2)
APB2   = 100 MHz (AHB / 2)
APB3   = 100 MHz (AHB / 2)
APB4   = 100 MHz (AHB / 2)
```

## Audio Clock Requirements

I2S audio requires specific clock relationships:

```
MCLK = 256 × Fs    (Master Clock)
BCLK = 64 × Fs     (Bit Clock, for stereo 32-bit)
LRCK = Fs          (Sample Rate)
```

| Sample Rate | MCLK | BCLK |
|-------------|------|------|
| 44.1 kHz | 11.2896 MHz | 2.8224 MHz |
| 48 kHz | 12.288 MHz | 3.072 MHz |
| 96 kHz | 24.576 MHz | 6.144 MHz |

**The Challenge**: These frequencies aren't nice multiples of 16 MHz. The PLL must generate them precisely.

## Audio Clock Generation (PLL3)

PLL3 is configured specifically for audio:

### For 48 kHz

```
HSE = 16 MHz
PLL3_M = 4      → 16 / 4 = 4 MHz (VCO input)
PLL3_N = 295    → 4 × 295 = 1180 MHz (VCO)
PLL3_P = 96     → 1180 / 96 = 12.2916... MHz ≈ MCLK
```

**But wait** - 12.2916 MHz isn't exactly 12.288 MHz. That's a 0.03% error, which accumulates over time.

### Fractional PLL

The STM32H750 supports **fractional-N PLL** for exact frequencies:

```
PLL3_N = 295.3125  (fractional!)
VCO = 4 × 295.3125 = 1181.25 MHz
MCLK = 1181.25 / 96 = 12.3046875 MHz
```

Still not perfect. For exact 48 kHz audio, configure:

```
PLL3_M = 4
PLL3_N = 384
PLL3_FRACN = 0
PLL3_P = 125

VCO = 4 × 384 = 1536 MHz
MCLK = 1536 / 125 = 12.288 MHz ✓ (exact!)
```

### For 96 kHz

Double the sample rate needs double MCLK:

```
PLL3_M = 4
PLL3_N = 384
PLL3_P = 62.5 → Not integer!
```

Use a different configuration:
```
PLL3_M = 5
PLL3_N = 192
PLL3_P = 25

VCO = (16/5) × 192 = 614.4 MHz
MCLK = 614.4 / 25 = 24.576 MHz ✓
```

## Clock Tree Diagram

```mermaid
graph TB
    subgraph Sources
        HSE[HSE Crystal<br/>16 MHz]
    end

    subgraph PLL1["PLL1 (System)"]
        P1M[/4] --> P1VCO[×400<br/>1600 MHz]
        P1VCO --> P1P[/4<br/>400 MHz]
    end

    subgraph PLL3["PLL3 (Audio)"]
        P3M[/4] --> P3VCO[×384<br/>1536 MHz]
        P3VCO --> P3P[/125<br/>12.288 MHz]
    end

    HSE --> P1M
    HSE --> P3M

    subgraph Outputs
        P1P --> SYSCLK[SYSCLK<br/>400 MHz]
        P3P --> SAI[SAI MCLK<br/>12.288 MHz]
        SYSCLK --> CPU[CPU<br/>400 MHz]
        SYSCLK --> AHB[AHB<br/>200 MHz]
    end
```

## Clock Configuration in Code

Using `stm32h7xx-hal`:

```rust
use stm32h7xx_hal::{pac, prelude::*, rcc};

let dp = pac::Peripherals::take().unwrap();

let pwr = dp.PWR.constrain();
let pwrcfg = pwr.freeze();

let rcc = dp.RCC.constrain();
let ccdr = rcc
    .sys_ck(400.MHz())        // System clock
    .pll1_strategy(rcc::PllConfigStrategy::Iterative)
    .pll3_p_ck(12_288_000.Hz())  // SAI MCLK for 48kHz
    .freeze(pwrcfg, &dp.SYSCFG);

// Verify clock
let sai_ck = ccdr.clocks.pll3_p_ck().unwrap();
assert_eq!(sai_ck.raw(), 12_288_000);
```

## Sample Rate and Buffer Timing

With properly configured clocks:

| Sample Rate | Buffer Size | Buffer Period | Processing Budget |
|-------------|-------------|---------------|-------------------|
| 48 kHz | 32 | 0.667 ms | 266,667 cycles |
| 48 kHz | 48 | 1.000 ms | 400,000 cycles |
| 48 kHz | 64 | 1.333 ms | 533,333 cycles |
| 96 kHz | 32 | 0.333 ms | 133,333 cycles |
| 96 kHz | 48 | 0.500 ms | 200,000 cycles |

**Processing Budget** = (Buffer Period) × (CPU Frequency)

At 400 MHz with 48 kHz / 48 samples:
- 400,000 cycles total
- ~8333 cycles per sample

## Clock Jitter

Clock jitter affects audio quality by varying sample timing:

```
Ideal:    |  |  |  |  |  |  |  |    (equal spacing)
Jitter:   | |  |   ||  | |   |  |   (irregular)
```

**Sources of Jitter**:
- PLL phase noise
- Power supply noise
- EMI coupling

**Mitigation**:
- Clean power supply
- Proper PCB layout
- Using external high-quality crystal

Daisy's design minimizes jitter for professional audio quality.

## Troubleshooting Clock Issues

### Audio Pitch is Wrong

**Symptom**: Everything plays at wrong pitch
**Cause**: MCLK frequency incorrect
**Fix**: Verify PLL3 configuration

### Audio Drifts Over Time

**Symptom**: Sync lost with external clock
**Cause**: PLL frequency slightly off
**Fix**: Use exact fractional PLL values

### Intermittent Audio Glitches

**Symptom**: Random clicks or dropouts
**Cause**: Clock domain crossing issues
**Fix**: Ensure SAI and DMA use same clock tree

### System Unstable at High Frequency

**Symptom**: Hard faults, random crashes
**Cause**: VCO out of valid range, voltage issues
**Fix**: Verify VCO stays in 192-960 MHz range

## Further Reading

- [Audio Interface](audio-interface.md) - How SAI uses these clocks
- [STM32H750 MCU](stm32h750.md) - Hardware overview
- [Troubleshooting](../troubleshooting.md) - Common issues
