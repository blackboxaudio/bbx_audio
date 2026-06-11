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
//! - 4-byte aligned via linker script
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
//! DMA1_STR1 interrupt priority is left at default. For custom priority,
//! set it before calling `init_and_start()` using `cortex_m::peripheral::Peripherals::take()`.
//!
//! ## Sample Format Conversion
//!
//! The codec uses unsigned 24-bit (u24) format:
//! - **I2S to f32**: `i32_to_f32()` converts u24 (0x000000-0xFFFFFF) to [-1.0, 1.0] via offset and normalization
//! - **f32 to I2S**: `f32_to_i32()` converts [-1.0, 1.0] to u24 format (0x000000 = -1.0, 0x800000 = 0.0, 0xFFFFFF =
//!   ~1.0)
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
//!     audio::set_callback(audio_callback);
//!     audio::start();
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

// ============================================================================
// Global State (interrupt-safe)
// ============================================================================

/// Global audio callback function pointer.
static AUDIO_CALLBACK: AtomicPtr<()> = AtomicPtr::new(default_callback as *mut ());

/// Flag indicating audio is running.
static AUDIO_RUNNING: AtomicBool = AtomicBool::new(false);

/// DMA transmit buffer (placed in DMA-accessible SRAM3, D2 domain). Cache-line (32-byte)
/// aligned by the linker so the optional `dcache` maintenance ops operate on whole lines.
#[unsafe(link_section = ".sram3")]
static mut TX_BUFFER: MaybeUninit<[u32; DMA_BUFFER_LENGTH]> = MaybeUninit::uninit();

/// DMA receive buffer (placed in DMA-accessible SRAM3, D2 domain).
#[unsafe(link_section = ".sram3")]
static mut RX_BUFFER: MaybeUninit<[u32; DMA_BUFFER_LENGTH]> = MaybeUninit::uninit();

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

/// Diagnostic latch for the DMA1 stream-1 interrupt-status flags captured on the **first**
/// `DMA1_STR1` IRQ (opt-in via the `diag_isr_oneshot` feature). Read by the `09_isr_oneshot`
/// example to tell an interrupt storm apart from a per-call hang in `process_audio_buffer`.
#[cfg(feature = "diag_isr_oneshot")]
pub static DIAG_DMA_FLAGS: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

/// Diagnostic counter incremented on every `DMA1_STR1` IRQ (opt-in via `diag_irq_rate`, which
/// also skips `process_audio_buffer`). The `10_irq_rate` example samples this over a 1 s window
/// to measure the DMA interrupt rate — distinguishing a too-fast SAI clock (rate ≫ ~1000/s)
/// from a too-slow `process_audio_buffer` (rate ≈ ~1000/s).
#[cfg(feature = "diag_irq_rate")]
pub static DIAG_IRQ_COUNT: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

/// Diagnostic latch for the cycle count of one `process_audio_buffer` call (opt-in via
/// `diag_process_time`). The `11_process_time` example converts this to microseconds to see
/// whether `process` is genuinely too slow (≥ the inter-IRQ period) or fast (paradox).
#[cfg(feature = "diag_process_time")]
pub static DIAG_PROCESS_CYCLES: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

/// Diagnostic: bitwise-OR of every sample the DMA has written into the RX buffer. Nonzero means
/// the codec's ADC is delivering data — i.e. the codec is powered, out of reset, and receiving a
/// usable MCLK. All-zero suggests the codec isn't running (no MCLK / still in reset). Opt-in via
/// `diag_peek`.
#[cfg(feature = "diag_peek")]
pub fn diag_rx_accumulate() -> u32 {
    let rx = unsafe { (*ptr::addr_of!(RX_BUFFER)).assume_init_ref() };
    rx.iter().fold(0u32, |acc, &s| acc | s)
}

// ============================================================================
// Public API
// ============================================================================

/// Set the audio callback function.
///
/// # Safety
///
/// Must be called before `start()` or while audio is stopped.
/// The callback must be realtime-safe (no allocations, no blocking).
pub fn set_callback(callback: AudioCallback) {
    // Safety: Only safe to call when audio is not running
    if !AUDIO_RUNNING.load(Ordering::SeqCst) {
        AUDIO_CALLBACK.store(callback as *mut (), Ordering::SeqCst);
    }
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
pub fn init_and_start(
    sample_rate: SampleRate,
    sai1: SAI1,
    dma1: DMA1,
    dma1_rec: rec::Dma1,
    sai1_pins: Sai1Pins,
    sai1_rec: rec::Sai1,
    clocks: &CoreClocks,
) {
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

    // Diagnostic: enable GPIOC clock + set PC7 as output so the DMA ISR can blink it
    // (opt-in via the `diag_led` feature). Raw register access keeps it independent of the
    // audio path's peripheral ownership. RCC_AHB4ENR.GPIOCEN (bit 2), GPIOC MODER pin 7.
    #[cfg(feature = "diag_led")]
    unsafe {
        const RCC_AHB4ENR: *mut u32 = 0x5802_44E0 as *mut u32;
        const GPIOC_MODER: *mut u32 = 0x5802_0800 as *mut u32;
        RCC_AHB4ENR.write_volatile(RCC_AHB4ENR.read_volatile() | (1 << 2));
        let _ = RCC_AHB4ENR.read_volatile();
        let moder = GPIOC_MODER.read_volatile();
        GPIOC_MODER.write_volatile((moder & !(0b11 << 14)) | (0b01 << 14));
    }

    // Initialize DMA buffers to zero using raw pointers
    let tx_buffer: &'static mut [u32; DMA_BUFFER_LENGTH] = unsafe {
        let tx_ptr = ptr::addr_of_mut!(TX_BUFFER);
        let buf = (*tx_ptr).assume_init_mut();
        buf.fill(0);
        buf
    };
    let rx_buffer: &'static mut [u32; DMA_BUFFER_LENGTH] = unsafe {
        let rx_ptr = ptr::addr_of_mut!(RX_BUFFER);
        let buf = (*rx_ptr).assume_init_mut();
        buf.fill(0);
        buf
    };

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

    // Enable DMA1 Stream 1 interrupt with high priority
    // Priority 0 = highest priority, prevents preemption by lower-priority interrupts
    // This ensures audio processing is not interrupted, reducing risk of buffer underruns
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::DMA1_STR1);
    }
    // Note: Priority setting requires mutable NVIC peripheral which is not available in this context.
    // The default priority is sufficient for most use cases. For custom priority, users can set it
    // before calling init_and_start() using cortex_m::peripheral::Peripherals::take().

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
    dma1_str0.start(|sai1_rb| {
        sai1.enable_dma(tx_channel);

        // Wait until SAI1's FIFO starts to receive data
        while sai1_rb.cha().sr.read().flvl().is_empty() {}

        sai1.enable();

        // Jump start audio - send first samples to get clocks running
        // This is required per the STM32H7 reference manual
        use stm32h7xx_hal::traits::i2s::FullDuplex;
        let _ = sai1.try_send(0, 0);
    });

    // Park the TX transfer so it is never dropped (its Drop disables the DMA stream, which would
    // silence the codec). Nothing reads it back — it just has to stay alive.
    unsafe {
        let tx_ptr = ptr::addr_of_mut!(DMA_TX_TRANSFER);
        (*tx_ptr).write(dma1_str0);
    }

    // Store the RX transfer handle for interrupt use
    unsafe {
        let transfer_ptr = ptr::addr_of_mut!(DMA_RX_TRANSFER);
        (*transfer_ptr).write(Some(dma1_str1));
    }

    AUDIO_RUNNING.store(true, Ordering::SeqCst);
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
    // Diagnostic heartbeat: toggle PC7 every ~250 buffers (~4 Hz) so we can see the audio
    // ISR is actually firing (i.e. SAI/DMA is streaming). Opt-in via the `diag_led` feature.
    #[cfg(feature = "diag_led")]
    {
        static DIAG: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
        if DIAG.fetch_add(1, Ordering::Relaxed) % 250 == 0 {
            const GPIOC_ODR: *mut u32 = 0x5802_0814 as *mut u32;
            unsafe { GPIOC_ODR.write_volatile(GPIOC_ODR.read_volatile() ^ (1 << 7)) };
        }
    }

    // Access buffers via raw pointers
    let tx_ptr = ptr::addr_of_mut!(TX_BUFFER);
    let rx_ptr = ptr::addr_of_mut!(RX_BUFFER);

    let tx_buffer = unsafe { (*tx_ptr).assume_init_mut() };
    let rx_buffer = unsafe { (*rx_ptr).assume_init_mut() };

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

/// Convert 24-bit I2S sample (unsigned u24 format in u32) to f32 [-1.0, 1.0].
///
/// The codec uses unsigned 24-bit format where:
/// - 0x000000 represents -1.0 (minimum)
/// - 0x800000 represents 0.0 (center)
/// - 0xFFFFFF represents ~1.0 (maximum)
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

/// Convert f32 [-1.0, 1.0] to 24-bit I2S sample (unsigned u24 format in u32).
///
/// The codec expects unsigned 24-bit format where:
/// - 0x000000 represents -1.0 (minimum)
/// - 0x800000 represents 0.0 (center)
/// - 0xFFFFFF represents ~1.0 (maximum)
///
/// This matches the reference daisy crate and libDaisy implementation.
#[inline(always)]
fn f32_to_i32(sample: f32) -> i32 {
    // Scale to 24-bit range and clamp
    let scaled = sample * 8_388_607.0;
    let clamped = scaled.clamp(-8_388_608.0, 8_388_607.0);

    // Convert to unsigned 24-bit format (cast to i32 then to u32)
    (clamped as i32) as u32 as i32
}

// ============================================================================
// Interrupt Handler
// ============================================================================

/// DMA1 Stream 1 interrupt handler (SAI1 RX half/complete).
#[interrupt]
fn DMA1_STR1() {
    // One-shot DMA diagnostic (opt-in): on the very first IRQ, latch the stream-1 interrupt
    // flags, then mask DMA1_STR1 so it can never re-fire. If `main` springs back to life
    // afterwards, the silence was an interrupt storm (now suppressed) and `process_audio_buffer`
    // is fine; if `main` stays frozen, `process_audio_buffer` hangs on its first call. Masking
    // here (not after processing) guarantees suppression even if the else-branch is taken.
    #[cfg(feature = "diag_isr_oneshot")]
    unsafe {
        const DMA1_LISR: *const u32 = 0x4002_0000 as *const u32;
        let lisr = DMA1_LISR.read_volatile();
        // Stream-1 flags: FEIF1=6, DMEIF1=8, TEIF1=9, HTIF1=10, TCIF1=11 (mask 0xF40).
        DIAG_DMA_FLAGS.store(lisr & 0xF40, Ordering::SeqCst);
        cortex_m::peripheral::NVIC::mask(pac::Interrupt::DMA1_STR1);
    }

    // Count every IRQ entry (opt-in) so `main` can measure the DMA interrupt rate and tell a
    // too-fast SAI clock apart from a too-slow process_audio_buffer.
    #[cfg(feature = "diag_irq_rate")]
    DIAG_IRQ_COUNT.fetch_add(1, Ordering::Relaxed);

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
        // (we write to the other half that's currently being DMA'd). The `diag_skip_process`
        // feature skips this so we can tell whether the IRQ itself storms (free-running DMA)
        // or whether process_audio_buffer is just too slow to finish within the IRQ period.
        #[cfg(not(any(
            feature = "diag_skip_process",
            feature = "diag_irq_rate",
            feature = "diag_process_time"
        )))]
        unsafe {
            process_audio_buffer(buffer_half);
        }
        // Time one process call with DWT.CYCCNT, latch it, then mask so `main` can report.
        #[cfg(feature = "diag_process_time")]
        unsafe {
            const DWT_CYCCNT: *const u32 = 0xE000_1004 as *const u32;
            let start = DWT_CYCCNT.read_volatile();
            process_audio_buffer(buffer_half);
            let end = DWT_CYCCNT.read_volatile();
            DIAG_PROCESS_CYCLES.store(end.wrapping_sub(start), Ordering::SeqCst);
            cortex_m::peripheral::NVIC::mask(pac::Interrupt::DMA1_STR1);
        }
        #[cfg(any(feature = "diag_skip_process", feature = "diag_irq_rate"))]
        let _ = buffer_half;
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
#[allow(dead_code)]
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
