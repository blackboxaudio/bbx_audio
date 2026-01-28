//! High-level external SDRAM driver for AS4C16M32MSA.
//!
//! Provides access to the 64MB on-board SDRAM with MPU configuration for proper caching.
//! Internal details (pin setup, FMC initialization, MPU configuration) are hidden from users.
//!
//! # Example
//!
//! ```ignore
//! // SDRAM is initialized via Board
//! let sdram = board.sdram();
//!
//! // Get base address for direct access
//! let base = sdram.base_address();
//!
//! // Allocate a slice from SDRAM
//! let buffer: &'static mut [f32] = unsafe {
//!     sdram.slice(0, 1024)
//! };
//! ```
//!
//! # Memory-Mapped Access
//!
//! After initialization, SDRAM is memory-mapped at address `0xC000_0000`.
//! You can also use the `.sdram_bss` linker section to place static buffers there.

#![allow(dead_code)]

use stm32_fmc::devices::as4c16m32msa_6;

use crate::{
    hal,
    hal::{fmc::FmcExt, gpio::Speed},
    pins::SdramPins,
};

/// SDRAM capacity in bytes (64MB).
const CAPACITY: usize = 64 * 1024 * 1024;

/// SDRAM base address (memory-mapped).
const BASE_ADDRESS: usize = 0xC000_0000;

// MPU register constants (ARMv7-M Architecture Reference Manual)
const MEMFAULTENA: u32 = 1 << 16;

/// External SDRAM (64MB AS4C16M32MSA).
///
/// Provides high-level access to the on-board 64MB SDRAM.
/// MPU is configured for write-back caching with write-through.
pub struct Sdram {
    base: *mut u32,
}

impl Sdram {
    /// Initialize the SDRAM.
    ///
    /// This is called internally by Board initialization. Users should access
    /// SDRAM via `board.sdram()` rather than calling this directly.
    ///
    /// # Arguments
    ///
    /// * `pins` - SDRAM pin configuration
    /// * `clocks` - System clocks reference
    /// * `fmc` - FMC peripheral
    /// * `fmc_rec` - FMC clock record
    /// * `mpu` - MPU peripheral (for cache configuration)
    /// * `scb` - SCB peripheral (for cache configuration)
    /// * `delay` - Delay provider for SDRAM initialization timing
    pub(crate) fn new<D: hal::hal::blocking::delay::DelayUs<u8>>(
        pins: SdramPins,
        clocks: &hal::rcc::CoreClocks,
        fmc: hal::pac::FMC,
        fmc_rec: hal::rcc::rec::Fmc,
        mpu: &mut hal::pac::MPU,
        scb: &mut hal::pac::SCB,
        delay: &mut D,
    ) -> Self {
        // Disable MPU while configuring
        disable_mpu(mpu, scb);

        // Initialize SDRAM via FMC
        let base_address = initialize_sdram(pins, clocks, fmc, fmc_rec, delay);

        // Configure MPU for SDRAM region
        configure_mpu_for_sdram(mpu, base_address);

        // Re-enable MPU
        enable_mpu(mpu, scb);

        Self { base: base_address }
    }

    /// Get the SDRAM base address.
    ///
    /// Returns a pointer to the start of SDRAM (0xC000_0000).
    #[inline]
    pub fn base_address(&self) -> *mut u32 {
        self.base
    }

    /// Get the SDRAM capacity in bytes.
    #[inline]
    pub fn capacity(&self) -> usize {
        CAPACITY
    }

    /// Allocate a slice from SDRAM.
    ///
    /// Returns a static mutable slice starting at the given byte offset.
    ///
    /// # Safety
    ///
    /// This is unsafe because:
    /// - The caller must ensure the offset and length don't exceed SDRAM bounds
    /// - The caller must ensure no other code accesses the same memory region
    /// - The returned slice has `'static` lifetime but is tied to hardware
    ///
    /// # Arguments
    ///
    /// * `offset` - Byte offset from SDRAM base address
    /// * `len` - Number of elements (not bytes) to allocate
    ///
    /// # Panics
    ///
    /// Panics if the requested region exceeds SDRAM bounds.
    #[inline]
    pub unsafe fn slice<T>(&self, offset: usize, len: usize) -> &'static mut [T] {
        let element_size = core::mem::size_of::<T>();
        let total_bytes = len * element_size;

        assert!(offset + total_bytes <= CAPACITY, "SDRAM allocation exceeds capacity");

        let ptr = (BASE_ADDRESS + offset) as *mut T;
        unsafe { core::slice::from_raw_parts_mut(ptr, len) }
    }

    /// Get the address at a given byte offset.
    ///
    /// # Panics
    ///
    /// Panics if offset exceeds SDRAM capacity.
    #[inline]
    pub fn address_at(&self, offset: usize) -> *mut u8 {
        assert!(offset < CAPACITY, "offset exceeds SDRAM capacity");
        (BASE_ADDRESS + offset) as *mut u8
    }
}

// Allow Sdram to be sent between threads (the underlying memory is thread-safe)
unsafe impl Send for Sdram {}

/// Initialize the SDRAM via FMC.
fn initialize_sdram<D: hal::hal::blocking::delay::DelayUs<u8>>(
    pins: SdramPins,
    clocks: &hal::rcc::CoreClocks,
    fmc: hal::pac::FMC,
    fmc_rec: hal::rcc::rec::Fmc,
    delay: &mut D,
) -> *mut u32 {
    // Configure all SDRAM pins with high-speed, pull-up, alternate function 12
    macro_rules! configure_pins {
        ($($pin:expr),*) => {
            (
                $(
                    $pin.into_push_pull_output()
                        .speed(Speed::VeryHigh)
                        .into_alternate::<12>()
                        .internal_pull_up(true)
                ),*
            )
        };
    }

    let sdram_pins = configure_pins! {
        pins.A0, pins.A1, pins.A2, pins.A3, pins.A4, pins.A5, pins.A6, pins.A7,
        pins.A8, pins.A9, pins.A10, pins.A11, pins.A12, pins.BA0, pins.BA1,
        pins.D0, pins.D1, pins.D2, pins.D3, pins.D4, pins.D5, pins.D6, pins.D7,
        pins.D8, pins.D9, pins.D10, pins.D11, pins.D12, pins.D13, pins.D14,
        pins.D15, pins.D16, pins.D17, pins.D18, pins.D19, pins.D20, pins.D21,
        pins.D22, pins.D23, pins.D24, pins.D25, pins.D26, pins.D27, pins.D28,
        pins.D29, pins.D30, pins.D31, pins.NBL0, pins.NBL1, pins.NBL2,
        pins.NBL3, pins.SDCKE0, pins.SDCLK, pins.SDNCAS, pins.SDNE0, pins.SDRAS,
        pins.SDNWE
    };

    // Initialize SDRAM using the AS4C16M32MSA device parameters
    fmc.sdram(sdram_pins, as4c16m32msa_6::As4c16m32msa {}, fmc_rec, clocks)
        .init(delay)
}

/// Configure MPU region 0 for SDRAM.
///
/// Cacheable, outer and inner write-back, no write allocate.
/// Reads are cached, writes go all the way to SDRAM.
fn configure_mpu_for_sdram(mpu: &mut hal::pac::MPU, base_address: *mut u32) {
    const REGION_NUMBER0: u32 = 0x00;
    const REGION_FULL_ACCESS: u32 = 0x03;
    const REGION_CACHEABLE: u32 = 0x01;
    const REGION_WRITE_BACK: u32 = 0x01;
    const REGION_ENABLE: u32 = 0x01;

    unsafe {
        mpu.rnr.write(REGION_NUMBER0);
        mpu.rbar.write((base_address as u32) & !0x1F);
        mpu.rasr.write(
            (REGION_FULL_ACCESS << 24)
                | (REGION_CACHEABLE << 17)
                | (REGION_WRITE_BACK << 16)
                | (log2minus1(CAPACITY as u32) << 1)
                | REGION_ENABLE,
        );
    }
}

/// Calculate log2(size) - 1 for MPU region size encoding.
fn log2minus1(sz: u32) -> u32 {
    for i in 5..=31 {
        if sz == (1 << i) {
            return i - 1;
        }
    }
    panic!("Invalid SDRAM size for MPU region");
}

/// Enable the MPU.
fn enable_mpu(mpu: &mut hal::pac::MPU, scb: &mut hal::pac::SCB) {
    const MPU_ENABLE: u32 = 0x01;
    const MPU_DEFAULT_MMAP_FOR_PRIVILEGED: u32 = 0x04;

    unsafe {
        mpu.ctrl.modify(|r| r | MPU_DEFAULT_MMAP_FOR_PRIVILEGED | MPU_ENABLE);

        scb.shcsr.modify(|r| r | MEMFAULTENA);

        // Ensure MPU settings take effect
        cortex_m::asm::dsb();
        cortex_m::asm::isb();
    }
}

/// Disable and reset the MPU.
fn disable_mpu(mpu: &mut hal::pac::MPU, scb: &mut hal::pac::SCB) {
    unsafe {
        // Make sure outstanding transfers are done
        cortex_m::asm::dmb();

        scb.shcsr.modify(|r| r & !MEMFAULTENA);

        // Disable the MPU and clear the control register
        mpu.ctrl.write(0);
    }
}
