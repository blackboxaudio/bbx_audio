//! SAI/I2S audio interface with DMA transfers.
//!
//! This module provides the audio interface for Daisy hardware using the
//! STM32H750's SAI peripheral in I2S master mode with circular DMA.
//!
//! # Architecture
//!
//! ## Clock Tree
//!
//! ```text
//! HSE (16 MHz external crystal)
//!   ├─> PLL1 → 480 MHz → SYSCLK (CPU, AHB, APB)
//!   └─> PLL3 → SAI MCLK:
//!       ├─> 12.288 MHz @ 48 kHz (256 × Fs)
//!       └─> 24.576 MHz @ 96 kHz (256 × Fs)
//! ```
//!
//! ## SAI Configuration
//!
//! Board-specific channel configurations:
//! - **seed/seed_1_2/pod**: TX on Channel A (master), RX on Channel B (slave)
//! - **seed_1_1/patch_sm**: TX on Channel B (slave), RX on Channel A (master)
//! - **Format**: 24-bit samples, MSB-justified, left-justified in 32-bit words
//!
//! ## DMA Configuration
//!
//! DMA stream assignment follows the SAI channel configuration:
//! - **seed/seed_1_2/pod**: Stream 0 → Channel A (TX), Stream 1 → Channel B (RX)
//! - **seed_1_1/patch_sm**: Stream 0 → Channel B (TX), Stream 1 → Channel A (RX)
//!
//! The DMA channels must match the SAI master/slave configuration to ensure audio
//! data flows correctly between memory buffers and the codec.
//!
//! ## Memory Layout
//!
//! ```text
//! DTCM (128KB):   0x20000000 - Stack, heap (fastest access)
//! AXI SRAM (512KB): 0x24000000 - General purpose RAM
//! SRAM1 (128KB):  0x30000000 - D2 domain, DMA-accessible
//! SRAM2 (128KB):  0x30020000 - D2 domain, DMA-accessible
//! SRAM3 (32KB):   0x30040000 - D2 domain, DMA buffers here ✓
//! SRAM4 (64KB):   0x38000000 - D3 domain, battery-backed
//! ```
//!
//! DMA audio buffers are placed in SRAM3 (D2 domain) which is:
//! - DMA-accessible by DMA1/DMA2
//! - Non-cached by default (no cache coherency issues)
//! - 32-byte (cache-line) aligned via `repr(align(32))` on the buffer type
//!
//! ## SAI Pin Configuration
//!
//! | Pin  | Function | Description                                    |
//! |------|----------|------------------------------------------------|
//! | PE2  | MCLK     | Master clock (12.288/24.576 MHz)               |
//! | PE4  | FS       | Frame sync (48/96 kHz)                         |
//! | PE5  | SCK      | Serial clock (3.072/6.144 MHz)                 |
//! | PE6  | SD_A     | Data A (TX: seed/seed_1_2/pod, RX: seed_1_1/patch_sm) |
//! | PE3  | SD_B     | Data B (RX: seed/seed_1_2/pod, TX: seed_1_1/patch_sm) |
//!
//! ## Interrupt Priority
//!
//! `init_and_start` sets DMA1_STR1 to mid priority 0x80 (H7 implements the
//! upper 4 bits; lower value = higher priority), leaving 0x00-0x70 free for
//! short, urgent user ISRs. To customize, set the priority again after
//! `init_and_start()` returns.
//!
//! ## Sample Format Conversion
//!
//! The codec speaks signed 24-bit two's-complement PCM in the low 24 bits of
//! each 32-bit slot:
//! - **I2S to f32**: `i32_to_f32()` sign-decodes the 24-bit word branchlessly (add 0x800000, mask, recenter) and
//!   normalizes to [-1.0, 1.0)
//! - **f32 to I2S**: `f32_to_i32()` scales, saturates, and truncates to the low 24 bits (two's complement)
//! - This matches the reference daisy crate and libDaisy implementation
//!
//! # Usage
//!
//! ```ignore
//! use bbx_daisy::{audio, FrameBuffer};
//!
//! fn audio_callback(input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
//!     // Process audio: copy input to output with some processing
//!     for i in 0..BLOCK_SIZE {
//!         let [left, right] = *input.frame(i);
//!         output.set_frame(i, left * 0.5, right * 0.5);
//!     }
//! }
//!
//! fn main() {
//!     // ... initialize hardware ...
//!     audio::set_callback(audio_callback).expect("audio already running");
//!     // audio::init_and_start(...) with the SAI/DMA peripherals — or use the
//!     // `bbx_daisy_audio!` macro, which wires all of this up for you.
//! }
//! ```

use core::{
    mem::MaybeUninit,
    ptr,
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};

use stm32h7xx_hal::{
    dma::{
        self, DBTransfer, MemoryToPeripheral, PeripheralToMemory, Transfer,
        dma::{DmaConfig, StreamsTuple},
    },
    gpio::{Alternate, gpioe},
    pac::{self, DMA1, SAI1, interrupt},
    prelude::*,
    rcc::{CoreClocks, rec},
    sai::{self, I2sUsers, SaiChannel, SaiI2sExt},
    time::Hertz,
};

use crate::{buffer::FrameBuffer, clock::SampleRate};

/// Block size for audio processing.
///
/// Default is 48 samples (~1ms latency at 48kHz).
/// Use `--features block_length_64` for 64 samples (~1.33ms latency).
#[cfg(not(feature = "block_length_64"))]
pub const BLOCK_SIZE: usize = 48;

/// Block size for audio processing (64 samples when block_length_64 feature is enabled).
#[cfg(feature = "block_length_64")]
pub const BLOCK_SIZE: usize = 64;

/// DMA buffer size in samples (double-buffered for ping-pong operation).
/// Format: [L, R, L, R, ...] with BLOCK_SIZE stereo frames * 2 halves
const DMA_BUFFER_LENGTH: usize = BLOCK_SIZE * 2 * 2;

/// Audio callback function type.
///
/// Called from the DMA interrupt with input samples and output buffer to fill.
/// Must complete within the buffer period (~1ms at 48kHz/48 samples).
pub type AudioCallback = fn(input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>);

/// Default passthrough callback (copies input to output).
fn default_callback(input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
    for i in 0..BLOCK_SIZE {
        let frame = input.frame(i);
        output.set_frame(i, frame[0], frame[1]);
    }
}

/// Errors surfaced by the audio interface API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioError {
    /// [`set_callback`] was called while audio is running.
    AlreadyRunning,
    /// The SAI FIFO never signalled readiness during [`init_and_start`] —
    /// typically a dead audio clock tree (PLL3) or wrong pin configuration.
    Timeout,
}

/// Iteration cap for the SAI-FIFO readiness spin in [`init_and_start`].
/// One audio frame (~20µs) suffices when the clocks are alive; this cap is
/// millisecond-scale at 480MHz, so hitting it means the clock tree is dead.
const SAI_FIFO_TIMEOUT_SPINS: u32 = 1_000_000;

// ============================================================================
// Global State (interrupt-safe)
// ============================================================================

/// Global audio callback function pointer.
static AUDIO_CALLBACK: AtomicPtr<()> = AtomicPtr::new(default_callback as *mut ());

/// Flag indicating audio is running.
static AUDIO_RUNNING: AtomicBool = AtomicBool::new(false);

/// DMA buffer storage, cache-line aligned by construction.
///
/// The 32-byte alignment must NOT depend on what else the linker happens to
/// place in `.sram3`: the `dcache` maintenance ops round outward to whole
/// cache lines, and a misaligned buffer would let them discard or write back
/// *adjacent* data mid-ISR. `repr(align(32))` guarantees the start address;
/// the const assert below guarantees the length.
#[repr(C, align(32))]
struct DmaBuffer([u32; DMA_BUFFER_LENGTH]);

const _: () = assert!(
    (DMA_BUFFER_LENGTH * core::mem::size_of::<u32>()) % 32 == 0,
    "DMA buffer size must be a whole number of 32-byte cache lines"
);

/// DMA transmit buffer (placed in DMA-accessible SRAM3, D2 domain).
#[unsafe(link_section = ".sram3")]
static mut TX_BUFFER: MaybeUninit<DmaBuffer> = MaybeUninit::uninit();

/// DMA receive buffer (placed in DMA-accessible SRAM3, D2 domain).
#[unsafe(link_section = ".sram3")]
static mut RX_BUFFER: MaybeUninit<DmaBuffer> = MaybeUninit::uninit();

/// Type alias for the DMA RX transfer.
#[cfg(not(any(feature = "seed_1_1", feature = "pod", feature = "patch_sm")))]
type DmaRxTransfer = Transfer<
    dma::dma::Stream1<DMA1>,
    sai::dma::ChannelB<SAI1>, // RX on Channel B for seed/seed_1_2 (codec TX on Channel A)
    PeripheralToMemory,
    &'static mut [u32; DMA_BUFFER_LENGTH],
    DBTransfer,
>;

#[cfg(any(feature = "seed_1_1", feature = "pod", feature = "patch_sm"))]
type DmaRxTransfer = Transfer<
    dma::dma::Stream1<DMA1>,
    sai::dma::ChannelA<SAI1>, // RX on Channel A for seed_1_1/pod/patch_sm (codec TX on Channel B)
    PeripheralToMemory,
    &'static mut [u32; DMA_BUFFER_LENGTH],
    DBTransfer,
>;

/// Global DMA transfer handle for interrupt access.
static mut DMA_RX_TRANSFER: MaybeUninit<Option<DmaRxTransfer>> = MaybeUninit::uninit();

/// Type alias for the DMA TX transfer (TX is always DMA1 stream 0; the SAI channel differs by
/// board). Mirrors `DmaRxTransfer`.
#[cfg(not(any(feature = "seed_1_1", feature = "pod", feature = "patch_sm")))]
type DmaTxTransfer = Transfer<
    dma::dma::Stream0<DMA1>,
    sai::dma::ChannelA<SAI1>, // TX on Channel A for seed/seed_1_2
    MemoryToPeripheral,
    &'static mut [u32; DMA_BUFFER_LENGTH],
    DBTransfer,
>;

#[cfg(any(feature = "seed_1_1", feature = "pod", feature = "patch_sm"))]
type DmaTxTransfer = Transfer<
    dma::dma::Stream0<DMA1>,
    sai::dma::ChannelB<SAI1>, // TX on Channel B for seed_1_1/pod/patch_sm
    MemoryToPeripheral,
    &'static mut [u32; DMA_BUFFER_LENGTH],
    DBTransfer,
>;

/// Holds the TX transfer alive for the lifetime of the program. Dropping a HAL `Transfer`
/// disables its DMA stream (see its `Drop` impl), so the TX transfer — which nothing else owns —
/// must be parked here, or the codec's DAC underruns into silence the moment `init_and_start`
/// returns. The ISR never touches it; it only needs to not be dropped.
static mut DMA_TX_TRANSFER: MaybeUninit<DmaTxTransfer> = MaybeUninit::uninit();

// ============================================================================
// Public API
// ============================================================================

/// Set the audio callback function.
///
/// Must be called before audio is started; the callback must be
/// realtime-safe (no allocations, no blocking).
///
/// # Errors
///
/// Returns [`AudioError::AlreadyRunning`] (without changing the callback) if
/// audio is already streaming.
pub fn set_callback(callback: AudioCallback) -> Result<(), AudioError> {
    if AUDIO_RUNNING.load(Ordering::SeqCst) {
        return Err(AudioError::AlreadyRunning);
    }
    AUDIO_CALLBACK.store(callback as *mut (), Ordering::SeqCst);
    Ok(())
}

/// Check if audio is currently running.
pub fn is_running() -> bool {
    AUDIO_RUNNING.load(Ordering::SeqCst)
}

/// Audio interface configuration.
pub struct AudioConfig {
    /// Sample rate (48kHz or 96kHz).
    pub sample_rate: SampleRate,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: SampleRate::Rate48000,
        }
    }
}

/// SAI1 pin set for audio I/O.
pub type Sai1Pins = (
    gpioe::PE2<Alternate<6>>,         // MCLK_A
    gpioe::PE5<Alternate<6>>,         // SCK_A
    gpioe::PE4<Alternate<6>>,         // FS_A
    gpioe::PE6<Alternate<6>>,         // SD_A (TX)
    Option<gpioe::PE3<Alternate<6>>>, // SD_B (RX)
);

/// Audio interface handle.
///
/// Manages SAI and DMA peripherals for audio I/O.
/// This struct is consumed by `start()` which takes ownership of the hardware.
pub struct AudioInterface {
    config: AudioConfig,
}

impl AudioInterface {
    /// Create a new audio interface with the given configuration.
    pub fn new(config: AudioConfig) -> Self {
        Self { config }
    }

    /// Get the configured sample rate.
    pub fn sample_rate(&self) -> SampleRate {
        self.config.sample_rate
    }

    /// Get the block size in samples.
    pub const fn block_size(&self) -> usize {
        BLOCK_SIZE
    }
}

/// Initialize and start the audio interface.
///
/// This function takes ownership of the necessary peripherals and starts
/// audio streaming. It configures:
///
/// - SAI1 in I2S mode (board-specific master/slave configuration)
/// - DMA1 streams 0/1 for TX/RX with circular buffers
/// - DMA interrupt for audio processing
///
/// # Arguments
///
/// * `sample_rate` - Audio sample rate (48kHz or 96kHz)
/// * `sai1` - SAI1 peripheral
/// * `dma1` - DMA1 peripheral
/// * `dma1_rec` - DMA1 clock configuration record
/// * `sai1_pins` - Configured SAI1 pins
/// * `sai1_rec` - SAI1 clock configuration record
/// * `clocks` - System clocks reference
///
/// The SAI channel/direction mapping is selected at compile time per board feature, since
/// stm32h7xx-hal types the SAI DMA channels (ChannelA/ChannelB) — the codec's TX/RX wiring
/// differs by Seed revision, so the board feature must match the hardware:
/// - TX on channel A (codec DAC on SD_A): AK4556 (`seed`), PCM3060 (`seed_1_2`)
/// - TX on channel B (codec DAC on SD_B): WM8731 (`seed_1_1`, `pod`), Patch SM (`patch_sm`)
///
/// # Errors
///
/// Returns [`AudioError::Timeout`] if the SAI FIFO never signals readiness —
/// typically a dead audio clock tree (PLL3) or wrong pin configuration.
pub fn init_and_start(
    sample_rate: SampleRate,
    sai1: SAI1,
    dma1: DMA1,
    dma1_rec: rec::Dma1,
    sai1_pins: Sai1Pins,
    sai1_rec: rec::Sai1,
    clocks: &CoreClocks,
) -> Result<(), AudioError> {
    // Optionally enable the CPU caches. The reference daisy crate (and libDaisy) run the audio
    // path with the D-cache ON, which is why `process_audio_buffer` invalidates/cleans the DMA
    // buffers — that maintenance is what keeps the cached CPU view coherent with DMA. It is gated
    // together with those ops behind the `dcache` feature: enabled here, maintained there.
    //
    // With `dcache` OFF (default) the D-cache stays disabled, the CPU and DMA both hit D2 SRAM
    // directly (already coherent), and no maintenance runs. This matters because calling the
    // SCB cache-maintenance-by-address ops with the D-cache *disabled* raises an imprecise
    // BusFault on this STM32H750 — so the two must be enabled/disabled as a pair.
    #[cfg(feature = "dcache")]
    unsafe {
        let mut cp = cortex_m::Peripherals::steal();
        cp.SCB.enable_icache();
        cp.SCB.enable_dcache(&mut cp.CPUID);
    }

    // Zero the NOLOAD buffer memory through raw pointers BEFORE creating any
    // reference: MaybeUninit's contract requires the value to be initialized
    // when assume_init_* runs.
    let tx_buffer: &'static mut [u32; DMA_BUFFER_LENGTH] = unsafe {
        let tx_ptr = ptr::addr_of_mut!(TX_BUFFER);
        core::ptr::write_bytes((*tx_ptr).as_mut_ptr(), 0, 1);
        &mut (*tx_ptr).assume_init_mut().0
    };
    let rx_buffer: &'static mut [u32; DMA_BUFFER_LENGTH] = unsafe {
        let rx_ptr = ptr::addr_of_mut!(RX_BUFFER);
        core::ptr::write_bytes((*rx_ptr).as_mut_ptr(), 0, 1);
        &mut (*rx_ptr).assume_init_mut().0
    };

    // Enforce the cache-line contract at runtime too (init-time, panics into
    // panic_halt before audio ever starts if the layout is broken).
    validate_dma_buffer(&tx_buffer[..]);
    validate_dma_buffer(&rx_buffer[..]);

    // Initialize global transfer holder using raw pointer
    unsafe {
        let transfer_ptr = ptr::addr_of_mut!(DMA_RX_TRANSFER);
        (*transfer_ptr).write(None);
    }

    // Configure DMA1 streams
    let dma1_streams = StreamsTuple::new(dma1, dma1_rec);

    // Configure DMA channel mapping based on the board's codec wiring (compile-time).
    #[cfg(not(any(feature = "seed_1_1", feature = "pod", feature = "patch_sm")))]
    let (tx_dma_channel, rx_dma_channel) = (
        unsafe { pac::Peripherals::steal().SAI1.dma_ch_a() }, // TX on Channel A (AK4556/PCM3060)
        unsafe { pac::Peripherals::steal().SAI1.dma_ch_b() }, // RX on Channel B
    );

    #[cfg(any(feature = "seed_1_1", feature = "pod", feature = "patch_sm"))]
    let (tx_dma_channel, rx_dma_channel) = (
        unsafe { pac::Peripherals::steal().SAI1.dma_ch_b() }, // TX on Channel B (WM8731/Patch SM)
        unsafe { pac::Peripherals::steal().SAI1.dma_ch_a() }, // RX on Channel A
    );

    // DMA1 Stream 0: TX (memory -> SAI1 Channel A/B depending on board)
    let dma_config = DmaConfig::default()
        .priority(dma::config::Priority::High)
        .memory_increment(true)
        .peripheral_increment(false)
        .circular_buffer(true)
        .fifo_enable(false);

    let mut dma1_str0: Transfer<_, _, MemoryToPeripheral, _, _> =
        Transfer::init(dma1_streams.0, tx_dma_channel, tx_buffer, None, dma_config);

    // DMA1 Stream 1: RX (SAI1 Channel B/A -> memory) with interrupts
    let dma_config = dma_config
        .transfer_complete_interrupt(true)
        .half_transfer_interrupt(true);

    let mut dma1_str1: Transfer<_, _, PeripheralToMemory, _, _> =
        Transfer::init(dma1_streams.1, rx_dma_channel, rx_buffer, None, dma_config);

    // Configure SAI1 for I2S: 24-bit, MSB-justified
    // Board-specific SAI channel configuration:
    // - seed/seed_1_2/pod: TX on Channel A (master), RX on Channel B (slave)
    // - seed_1_1/patch_sm: TX on Channel B (slave), RX on Channel A (master)
    #[cfg(not(any(feature = "seed_1_1", feature = "pod", feature = "patch_sm")))]
    let (tx_is_master, rx_sync_type) = (true, sai::I2SSync::Internal);

    #[cfg(any(feature = "seed_1_1", feature = "pod", feature = "patch_sm"))]
    let (tx_is_master, rx_sync_type) = (false, sai::I2SSync::Internal);

    let sai1_tx_config = if tx_is_master {
        sai::I2SChanConfig::new(sai::I2SDir::Tx)
            .set_frame_sync_active_high(true)
            .set_clock_strobe(sai::I2SClockStrobe::Falling)
    } else {
        sai::I2SChanConfig::new(sai::I2SDir::Tx)
            .set_sync_type(rx_sync_type)
            .set_frame_sync_active_high(true)
            .set_clock_strobe(sai::I2SClockStrobe::Falling)
    };

    let sai1_rx_config = if !tx_is_master {
        sai::I2SChanConfig::new(sai::I2SDir::Rx)
            .set_frame_sync_active_high(true)
            .set_clock_strobe(sai::I2SClockStrobe::Rising)
    } else {
        sai::I2SChanConfig::new(sai::I2SDir::Rx)
            .set_sync_type(rx_sync_type)
            .set_frame_sync_active_high(true)
            .set_clock_strobe(sai::I2SClockStrobe::Rising)
    };

    // Use the provided sample rate (PLL3 MCLK must match: 12.288MHz @ 48kHz, 24.576MHz @ 96kHz)
    let sample_rate_hz = Hertz::from_raw(sample_rate.hz());

    // Channel A is the primary (master) block: its config is the TX direction when the codec
    // transmits on channel A (AK4556/PCM3060), otherwise the RX direction (WM8731/Patch SM).
    #[cfg(not(any(feature = "seed_1_1", feature = "pod", feature = "patch_sm")))]
    let sai1_users = I2sUsers::new(sai1_tx_config).add_slave(sai1_rx_config);

    #[cfg(any(feature = "seed_1_1", feature = "pod", feature = "patch_sm"))]
    let sai1_users = I2sUsers::new(sai1_rx_config).add_slave(sai1_tx_config);

    let mut sai1 = sai1.i2s_ch_a(
        sai1_pins,
        sample_rate_hz,
        sai::I2SDataSize::BITS_24,
        sai1_rec,
        clocks,
        sai1_users,
    );

    // NOTE: the DMA1_STR1 interrupt is NOT unmasked here. It is unmasked at the
    // end of this function, after the transfer handles are stored — see below.

    // Determine which channels to enable based on the board's codec wiring (compile-time).
    #[cfg(not(any(feature = "seed_1_1", feature = "pod", feature = "patch_sm")))]
    let (tx_channel, rx_channel) = (SaiChannel::ChannelA, SaiChannel::ChannelB);

    #[cfg(any(feature = "seed_1_1", feature = "pod", feature = "patch_sm"))]
    let (tx_channel, rx_channel) = (SaiChannel::ChannelB, SaiChannel::ChannelA);

    // Start RX DMA first (enables the RX channel for the configured board)
    dma1_str1.start(|_sai1_rb| {
        sai1.enable_dma(rx_channel);
    });

    // Start TX DMA and enable SAI
    let mut fifo_timed_out = false;
    dma1_str0.start(|sai1_rb| {
        sai1.enable_dma(tx_channel);

        // Bounded wait until SAI1's FIFO starts to receive data. Unbounded,
        // a dead clock tree or wrong pin config would hang boot silently.
        let mut spins: u32 = 0;
        while sai1_rb.cha().sr.read().flvl().is_empty() {
            spins += 1;
            if spins > SAI_FIFO_TIMEOUT_SPINS {
                fifo_timed_out = true;
                return;
            }
        }

        sai1.enable();

        // Jump start audio - send first samples to get clocks running
        // This is required per the STM32H7 reference manual
        use stm32h7xx_hal::traits::i2s::FullDuplex;
        let _ = sai1.try_send(0, 0);
    });
    if fifo_timed_out {
        return Err(AudioError::Timeout);
    }

    // Park the TX transfer so it is never dropped (its Drop disables the DMA stream, which would
    // silence the codec). Nothing reads it back — it just has to stay alive.
    unsafe {
        let tx_ptr = ptr::addr_of_mut!(DMA_TX_TRANSFER);
        (*tx_ptr).write(dma1_str0);
    }

    // Store the RX transfer handle for interrupt use. This MUST happen before
    // the NVIC unmask below: DMA HT/TC events latch while the interrupt is
    // masked and are serviced immediately on unmask — but an ISR that ran
    // before this store would find `None` (see the defensive branch in
    // `DMA1_STR1` for why that formerly meant a livelock).
    unsafe {
        let transfer_ptr = ptr::addr_of_mut!(DMA_RX_TRANSFER);
        (*transfer_ptr).write(Some(dma1_str1));
    }

    // Make the handle stores visible before the interrupt can observe them.
    core::sync::atomic::compiler_fence(Ordering::SeqCst);

    // Set an explicit mid priority for the audio interrupt, then unmask it.
    // The H7 implements the upper 4 priority bits (16 levels; lower value =
    // higher priority). 0x80 keeps audio above default-priority interrupts
    // while leaving 0x00-0x70 free for short, truly urgent ISRs a user may
    // add (UART RX, encoders) — at reset priority 0 the ~1ms audio callback
    // would starve everything else for a full block period.
    // Init-time steal(): single-threaded, pre-audio — same pattern as the
    // dcache setup above.
    unsafe {
        let mut cp = cortex_m::Peripherals::steal();
        cp.NVIC.set_priority(pac::Interrupt::DMA1_STR1, 0x80);
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::DMA1_STR1);
    }

    AUDIO_RUNNING.store(true, Ordering::SeqCst);

    Ok(())
}

// ============================================================================
// DMA Interrupt Handler
// ============================================================================

/// Process audio at the given buffer half (0 = first half, 1 = second half).
///
/// Called from DMA interrupt on half-transfer and transfer-complete events.
///
/// # Safety
///
/// Must only be called from the DMA interrupt handler.
#[inline(always)]
unsafe fn process_audio_buffer(buffer_half: usize) {
    // Access buffers via raw pointers
    let tx_ptr = ptr::addr_of_mut!(TX_BUFFER);
    let rx_ptr = ptr::addr_of_mut!(RX_BUFFER);

    let tx_buffer = unsafe { &mut (*tx_ptr).assume_init_mut().0 };
    let rx_buffer = unsafe { &mut (*rx_ptr).assume_init_mut().0 };

    // Invalidate D-cache for RX buffer before reading so we see the DMA's writes, not stale
    // cached data. Only needed (and only safe) when the D-cache is enabled — see `dcache` in
    // `init_and_start`. With the cache off the buffer is read directly from SRAM (coherent).
    #[cfg(feature = "dcache")]
    unsafe {
        cortex_m::Peripherals::steal().SCB.invalidate_dcache_by_slice(rx_buffer);
    }

    let stereo_block_length = BLOCK_SIZE * 2; // L, R pairs
    let offset = buffer_half * stereo_block_length;

    // Convert DMA samples (i32 in u32) to f32 for processing
    let mut input: FrameBuffer<BLOCK_SIZE> = FrameBuffer::new();
    let mut output: FrameBuffer<BLOCK_SIZE> = FrameBuffer::new();

    // Deinterleave and convert RX buffer to FrameBuffer
    for i in 0..BLOCK_SIZE {
        let left_u32 = rx_buffer[offset + i * 2];
        let right_u32 = rx_buffer[offset + i * 2 + 1];
        let left = i32_to_f32(left_u32 as i32);
        let right = i32_to_f32(right_u32 as i32);
        input.set_frame(i, left, right);
    }

    // Call user callback (loaded atomically)
    let callback_ptr = AUDIO_CALLBACK.load(Ordering::SeqCst);
    let callback: AudioCallback = unsafe { core::mem::transmute(callback_ptr) };
    callback(&input, &mut output);

    // Convert and interleave output FrameBuffer to TX buffer
    for i in 0..BLOCK_SIZE {
        let [left, right] = *output.frame(i);
        tx_buffer[offset + i * 2] = f32_to_i32(left) as u32;
        tx_buffer[offset + i * 2 + 1] = f32_to_i32(right) as u32;
    }

    // Clean D-cache for TX buffer after writing so the DMA reads our samples, not stale cache.
    // Only needed/safe with the D-cache enabled (see `dcache`); with the cache off the CPU writes
    // straight to SRAM that the DMA reads (coherent).
    #[cfg(feature = "dcache")]
    unsafe {
        cortex_m::Peripherals::steal().SCB.clean_dcache_by_slice(tx_buffer);
    }
}

/// Convert a 24-bit I2S sample (signed two's complement in the low 24 bits)
/// to f32 in [-1.0, 1.0).
///
/// The `+0x800000` / mask / `-1.0` sequence is a branchless sign-decode: it
/// maps 0x000000..=0x7FFFFF to [0.0, 1.0) and 0x800000..=0xFFFFFF (negative
/// two's-complement values) to [-1.0, 0.0) — equivalent to sign extension.
///
/// This matches the reference daisy crate and libDaisy implementation.
#[inline(always)]
fn i32_to_f32(sample: i32) -> f32 {
    use core::num::Wrapping;

    // Convert to unsigned 24-bit by adding 0x800000 (center point)
    let y = sample as u32;
    let y = (Wrapping(y) + Wrapping(0x0080_0000)).0 & 0x00FF_FFFF;

    // Normalize to [-1.0, 1.0] range
    (y as f32 / 8_388_608.0) - 1.0
}

/// Convert f32 [-1.0, 1.0] to a 24-bit I2S sample (signed two's complement).
///
/// Scales to the 24-bit range and saturates (Rust float→int casts saturate;
/// NaN becomes 0). The SAI transmits only the low 24 bits of the word, which
/// are the correct two's-complement representation for all in-range values.
///
/// This matches the reference daisy crate and libDaisy implementation.
#[inline(always)]
fn f32_to_i32(sample: f32) -> i32 {
    // Scale to 24-bit range and clamp
    let scaled = sample * 8_388_607.0;
    let clamped = scaled.clamp(-8_388_608.0, 8_388_607.0);

    clamped as i32
}

// ============================================================================
// Interrupt Handler
// ============================================================================

/// DMA1 Stream 1 interrupt handler (SAI1 RX half/complete).
#[interrupt]
fn DMA1_STR1() {
    // Safety: We only access this from the interrupt handler
    let transfer_ptr = ptr::addr_of_mut!(DMA_RX_TRANSFER);
    let transfer = unsafe { (*transfer_ptr).assume_init_mut() };

    if let Some(transfer) = transfer {
        let buffer_half = if transfer.get_half_transfer_flag() {
            transfer.clear_half_transfer_interrupt();
            0
        } else if transfer.get_transfer_complete_flag() {
            transfer.clear_transfer_complete_interrupt();
            1
        } else {
            return;
        };

        // Process audio in the half that was just filled
        // (we write to the other half that's currently being DMA'd).
        unsafe {
            process_audio_buffer(buffer_half);
        }
    } else {
        // Defensive: with the handle stored before the NVIC unmask this branch
        // should be unreachable — but if it ever runs, clear the stream-1
        // flags so a latched event cannot re-enter the ISR forever (livelock).
        let dma1 = unsafe { &*DMA1::ptr() };
        dma1.lifcr.write(|w| {
            w.cfeif1()
                .set_bit()
                .cdmeif1()
                .set_bit()
                .cteif1()
                .set_bit()
                .chtif1()
                .set_bit()
                .ctcif1()
                .set_bit()
        });
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Create an audio interface with default configuration (48kHz).
pub fn default_audio() -> AudioInterface {
    AudioInterface::new(AudioConfig::default())
}

/// Create an audio interface with specified sample rate.
pub fn audio_with_rate(sample_rate: SampleRate) -> AudioInterface {
    AudioInterface::new(AudioConfig { sample_rate })
}

/// Default sample rate as f32 for DSP calculations.
#[cfg(not(feature = "sampling_rate_96khz"))]
pub const DEFAULT_SAMPLE_RATE: f32 = 48_000.0;

/// Default sample rate as f32 for DSP calculations (96kHz when sampling_rate_96khz feature is enabled).
#[cfg(feature = "sampling_rate_96khz")]
pub const DEFAULT_SAMPLE_RATE: f32 = 96_000.0;

// ============================================================================
// Cache Line Validation
// ============================================================================

/// Cache line size for STM32H7 (32 bytes).
const CACHE_LINE_SIZE: usize = 32;

/// Validate that a DMA buffer is properly aligned for cache operations.
///
/// DMA buffers must be cache-line aligned (32 bytes on STM32H7) to avoid
/// data corruption when invalidating/cleaning cache around DMA transfers.
///
/// # Panics
///
/// Panics if the buffer is not properly aligned or if its size is not a
/// multiple of the cache line size.
pub(crate) fn validate_dma_buffer<T>(buffer: &[T]) {
    let ptr = buffer.as_ptr() as usize;
    let size = core::mem::size_of_val(buffer);

    assert!(
        ptr % CACHE_LINE_SIZE == 0,
        "DMA buffer must be cache-line aligned (32 bytes)"
    );
    assert!(
        size % CACHE_LINE_SIZE == 0,
        "DMA buffer size must be a multiple of cache line size (32 bytes)"
    );
}
