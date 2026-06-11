# Audio Debug Handoff — silent audio on Daisy (bbx_daisy SAI/DMA bring-up)

> Hand this to a fresh Claude instance (or engineer) to continue diagnosing why audio is
> silent on the Daisy. The DSP, build, and flash pipeline all work — the **audio engine
> (SAI + DMA) has never produced sound on hardware**. Work iteratively: **you cannot flash
> hardware**; propose diagnostics, the user flashes and reports what the LED does.

## Hardware (confirmed)
- **Daisy Pod, 2022 Rev5**, containing an **original Daisy Seed → AK4556 codec** (confirmed at
  runtime: the PD3/PD4 strap pins both read high → libDaisy `DAISY_SEED`). So: **AK4556**, the
  I2S-only codec with **no I2C** — it auto-detects sample rate from MCLK/LRCK and only needs a
  reset pulse on **PB11**. SAI data direction for AK4556: **TX on SAI1 channel A**, RX on B.
- STM32H750, 16 MHz HSE. Audio target: 48 kHz, 24-bit, block size 48.

## Environment
- Repo: `~/dev/bbx/repos/bbx_audio`; crate: `bbx_daisy/`.
- **Build from inside `bbx_daisy/`** (its `.cargo/config.toml` sets the `thumbv7em-none-eabihf`
  target and a `dfu-util` runner). Example: `cd bbx_daisy && cargo run --example NAME --release`.
- Flash is **DFU** (`dfu-util` installed; **no probe-rs, no debug probe, no RTT/defmt**). The
  Daisy enters DFU mode via hold BOOT + tap RESET. The user has `arm-none-eabi-*` tools.
- **The only observable output is the onboard LED on PC7** (the Seed's user LED — confirmed
  visible/working). All diagnostics must surface state via PC7 blink patterns.
- Toolchain: pinned nightly (`rust-toolchain.toml`); `cargo fmt`/`clippy` use it automatically.

## Reference = libDaisy (the ground truth)
A full C++ HAL for the same hardware is checked out at
`~/dev/bbx/repos/flora/vendor/libDaisy/src/`. The bbx_daisy audio code was written *without*
verifying against it and has already had several bugs found this way. Key files:
- `daisy_seed.cpp` → `ConfigureAudio()` (SAI config per codec), `CheckBoardVersion()`.
- `per/sai.{h,cpp}` → the SAI/I2S setup and DMA.
- `dev/codec_ak4556.{h,cpp}` → AK4556 reset sequence.

**Also compare against known-good *Rust* Daisy audio** built on the same `stm32h7xx-hal` SAI
API: the `daisy` crate (github: zlosynth/daisy), `antoinevg/daisy_bsp`, and `libdaisy-rust`.
Diffing bbx_daisy's SAI/DMA setup against one of these is likely the fastest path to the fix.

## What's CONFIRMED WORKING (bisection so far)
1. **Build + DFU flash** — `01_blink` runs (LED blinks).
2. **GPIO / PC7 LED** — works.
3. **Audio clock** — `07_clock_check` example runs the full audio clock config
   (`ClockConfig::configure`: VOS0 + PLL3 fractional → PLL3_P = 12.288 MHz for the SAI MCLK)
   then fast-blinks PC7. **It fast-blinks** → the clock `freeze()` succeeds and execution
   continues. (Note: the simpler non-audio `Board::init` clock uses 400 MHz **without** VOS0;
   the audio clock adds VOS0 + PLL3.)
4. **Codec identity** — `06_board_id` example reads PD3/PD4 and blinks a count: **1 blink =
   AK4556**.

## What's BROKEN (the symptom)
Every audio example is silent. The latest test: `02_oscillator` built with the **`diag_led`**
feature (which toggles PC7 from inside the DMA RX interrupt, ~every 250 buffers ≈ 2 Hz):
```bash
cd bbx_daisy && cargo run --example 02_oscillator --features diag_led --release
```
**Result: PC7 is CONSTANTLY LIT (not blinking, not dark), and no audio.**

Interpretation: `dark` would mean the ISR never fires; a clean ~2 Hz blink would mean it fires
normally. "Constantly lit" means **neither** — most likely:
- **(a) interrupt storm**: a DMA error flag (TEIF/FEIF/DMEIF) is set and the ISR's fallthrough
  `else { return }` doesn't clear it, so it re-fires continuously (toggle too fast to see) and
  starves `main`; or
- **(b) the DMA does one transfer then stalls** (PC7 toggled once → stuck high).

Both mean the SAI/DMA is not streaming continuously.

## The audio code path (where to look)
- `bbx_daisy/src/macros.rs` — `bbx_daisy_audio!` builds the entry point: writes the processor,
  calls `board::init_audio()`, calls `AudioProcessor::prepare()`, `audio::set_callback()`,
  `audio::init_and_start(...)`, then `loop { wfi() }`. (Pod-with-knobs variant:
  `bbx_daisy_audio_with_controls!` → `board::init_audio_with_adc()`.)
- `bbx_daisy/src/board.rs` — `init_audio()`: clock config, **AK4556 PB11 reset pulse**
  (set_low, ~1 ms, set_high, ~1 ms), SAI1 pins (PE2 MCLK, PE5 SCK, PE4 FS, PE6 SD_A/TX,
  PE3 SD_B/RX, all AF6), returns `AudioPeripherals`. Codec selected at compile time per board
  feature (the SAI DMA channels are typed, so runtime codec detection isn't feasible without
  enum-wrapping the transfers).
- `bbx_daisy/src/audio.rs` — **the prime suspect.** `init_and_start()` sets up SAI1 (I2S,
  24-bit; for AK4556: ch A master TX / ch B slave RX), DMA1 stream 0 (TX) + stream 1 (RX,
  circular, HT+TC interrupts), the `sai1.i2s_ch_a(...)` call with `I2sUsers`, then a "jump
  start": `dma1_str1.start(...)`, `dma1_str0.start(|rb| { enable ch; while fifo empty {}; sai.enable(); sai.try_send(0,0) })`,
  unmasks `DMA1_STR1`, stores the RX transfer in a global. The ISR:
  ```rust
  #[interrupt]
  fn DMA1_STR1() {
      let transfer = unsafe { (*ptr::addr_of_mut!(DMA_RX_TRANSFER)).assume_init_mut() };
      if let Some(transfer) = transfer {
          let buffer_half = if transfer.get_half_transfer_flag() {
              transfer.clear_half_transfer_interrupt(); 0
          } else if transfer.get_transfer_complete_flag() {
              transfer.clear_transfer_complete_interrupt(); 1
          } else {
              return;   // <-- does NOT clear error flags (TE/FE/DME) -> possible storm
          };
          unsafe { process_audio_buffer(buffer_half); }
      }
  }
  ```
  `process_audio_buffer()` invalidates D-cache on RX, converts u24<->f32, calls the user
  callback, cleans D-cache on TX. DMA buffers live in `.sram3` (D2 domain) — see `memory.x`.
- `bbx_daisy/src/clock.rs` — `ClockConfig` (confirmed working). `bbx_daisy/src/init.rs` —
  `#[pre_init]` (FPU, RCC reset, **D2-domain SRAM clocks** for the DMA buffers, AXI SRAM errata).

## Suspects, roughly ranked
1. **DMA error / interrupt storm** — confirm first. Are error interrupts generated and never
   cleared? Is the ISR's `else { return }` the culprit? (Matches the "constant lit" symptom.)
2. **SAI configuration mismatch vs a known-good `stm32h7xx-hal` Daisy setup** — master/slave,
   sync, `I2sUsers::new(tx).add_slave(rx)` ordering, clock strobe, MCLK enable, data size.
3. **MCLK not actually reaching the codec** — clock `freeze()` succeeds, but is SAI1 MCLK (PE2)
   really output at 12.288 MHz? AK4556 needs MCLK to come alive.
4. **AK4556 reset (PB11)** sequence/timing vs `dev/codec_ak4556.cpp`.
5. **DMA buffer cache coherency / placement** (SRAM3, D-cache maintenance, `.sram3` section).
6. The "jump start" (`try_send(0,0)`, FIFO-wait loop) correctness.

## How to drive the diagnosis (the working method)
Build small LED-on-PC7 diagnostics, have the user flash + report, bisect. Patterns to add:
- **Reach-the-main-loop check**: blink PC7 *slowly from `main`'s loop* (not the ISR). If it
  blinks slowly → `main` runs and the ISR is NOT storming (so audio just isn't producing output
  → suspect SAI/codec/MCLK). If `main` never blinks → ISR is storming / init hangs.
- **DMA-flag reporter**: in the ISR, read the DMA stream's status flags and blink a *count* of
  which flag is set (HT vs TC vs TE vs FE vs DME) to confirm/deny an error storm; then clear ALL
  flags. Or read the SAI `SR` register.
- **Stage tracer**: toggle PC7 at successive points inside `init_and_start` to see how far init
  gets before the storm/stall.

Keep diagnostics behind features or as separate `0X_*.rs` examples using the **host-stub
pattern**: gate the body on `#[cfg(all(target_arch="arm", target_os="none"))] mod app { ... }`
with a `#[cfg(not(...))] fn main(){}` fallback, since the workspace also builds on the host (see
existing examples `01_blink.rs` … `07_clock_check.rs`). Existing diagnostics: `06_board_id`,
`07_clock_check`, and the `diag_led` feature in `audio.rs`.

## Build / flash quick reference
```bash
cd ~/dev/bbx/repos/bbx_audio/bbx_daisy

# Build/flash an example (seed/AK4556 is the default feature):
cargo run --example 02_oscillator --release            # DFU mode first: hold BOOT, tap RESET
cargo run --example 02_oscillator --features diag_led --release

# Manual flash of any built ELF:
./scripts/flash-dfu.sh ../target/thumbv7em-none-eabihf/release/examples/<name>
```

## Constraints / etiquette
- You can't flash — every hardware claim must be verified by the user via a PC7 pattern.
- No RTT/defmt/probe; PC7 LED is the only output channel.
- `cargo build`/`clippy`/`fmt` from inside `bbx_daisy/` for the ARM target; the host build must
  keep compiling too (examples use the `mod app` + host-stub `#[cfg]` pattern).
- The whole audio engine is unverified, so treat libDaisy and a known-good Rust Daisy BSP as the
  source of truth and be willing to rewrite `init_and_start` to match one of them.

## Start here
1. Read `bbx_daisy/src/audio.rs` (`init_and_start`, `DMA1_STR1`, `process_audio_buffer`) in full.
2. Propose a single PC7 diagnostic that distinguishes **"interrupt storm"** from **"main loop
   running but no audio output."** (E.g. a slow heartbeat from `main`'s loop.)
3. Confirm that hypothesis with the user before changing any SAI/DMA code.
4. In parallel, diff `init_and_start` against zlosynth/`daisy` or `antoinevg/daisy_bsp` — the
   SAI/DMA setup is much faster to fix by comparison with a known-good `stm32h7xx-hal` impl than
   from first principles.
