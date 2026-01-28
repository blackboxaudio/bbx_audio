//! Pin mappings for Daisy hardware variants.
//!
//! Each board variant has its own pin mapping module that defines
//! the GPIO assignments for audio, peripherals, and user I/O.

#![allow(unused_imports)]

use stm32h7xx_hal::gpio;

#[cfg(any(feature = "seed", feature = "seed_1_1", feature = "seed_1_2"))]
pub mod seed;

#[cfg(feature = "pod")]
pub mod pod;

#[cfg(feature = "patch_sm")]
pub mod patch_sm;

#[cfg(feature = "patch_sm")]
pub use patch_sm::*;
#[cfg(feature = "pod")]
pub use pod::*;
#[cfg(any(feature = "seed", feature = "seed_1_1", feature = "seed_1_2"))]
pub use seed::*;

// ============================================================================
// Internal Peripheral Pin Structures
// ============================================================================
// These are internal pin groupings for flash/SDRAM peripherals.
// Users interact with high-level types (Flash, Sdram) instead.

/// QSPI Flash pins for IS25LP064 (internal use only).
#[allow(non_snake_case, dead_code)]
pub(crate) struct QspiFlashPins {
    pub IO0: gpio::gpiof::PF8<gpio::Analog>,  // QUADSPI_BK1_IO0 (SI)
    pub IO1: gpio::gpiof::PF9<gpio::Analog>,  // QUADSPI_BK1_IO1 (SO)
    pub IO2: gpio::gpiof::PF7<gpio::Analog>,  // QUADSPI_BK1_IO2
    pub IO3: gpio::gpiof::PF6<gpio::Analog>,  // QUADSPI_BK1_IO3
    pub SCK: gpio::gpiof::PF10<gpio::Analog>, // QUADSPI_CLK
    pub CS: gpio::gpiog::PG6<gpio::Analog>,   // QUADSPI_BK1_NCS
}

/// SDRAM pins for AS4C16M32MSA (internal use only).
#[allow(non_snake_case, dead_code)]
pub(crate) struct SdramPins {
    // Address lines A0-A12
    pub A0: gpio::gpiof::PF0<gpio::Analog>,
    pub A1: gpio::gpiof::PF1<gpio::Analog>,
    pub A2: gpio::gpiof::PF2<gpio::Analog>,
    pub A3: gpio::gpiof::PF3<gpio::Analog>,
    pub A4: gpio::gpiof::PF4<gpio::Analog>,
    pub A5: gpio::gpiof::PF5<gpio::Analog>,
    pub A6: gpio::gpiof::PF12<gpio::Analog>,
    pub A7: gpio::gpiof::PF13<gpio::Analog>,
    pub A8: gpio::gpiof::PF14<gpio::Analog>,
    pub A9: gpio::gpiof::PF15<gpio::Analog>,
    pub A10: gpio::gpiog::PG0<gpio::Analog>,
    pub A11: gpio::gpiog::PG1<gpio::Analog>,
    pub A12: gpio::gpiog::PG2<gpio::Analog>,
    // Bank address lines
    pub BA0: gpio::gpiog::PG4<gpio::Analog>,
    pub BA1: gpio::gpiog::PG5<gpio::Analog>,
    // Data lines D0-D31 (32-bit bus)
    pub D0: gpio::gpiod::PD14<gpio::Analog>,
    pub D1: gpio::gpiod::PD15<gpio::Analog>,
    pub D2: gpio::gpiod::PD0<gpio::Analog>,
    pub D3: gpio::gpiod::PD1<gpio::Analog>,
    pub D4: gpio::gpioe::PE7<gpio::Analog>,
    pub D5: gpio::gpioe::PE8<gpio::Analog>,
    pub D6: gpio::gpioe::PE9<gpio::Analog>,
    pub D7: gpio::gpioe::PE10<gpio::Analog>,
    pub D8: gpio::gpioe::PE11<gpio::Analog>,
    pub D9: gpio::gpioe::PE12<gpio::Analog>,
    pub D10: gpio::gpioe::PE13<gpio::Analog>,
    pub D11: gpio::gpioe::PE14<gpio::Analog>,
    pub D12: gpio::gpioe::PE15<gpio::Analog>,
    pub D13: gpio::gpiod::PD8<gpio::Analog>,
    pub D14: gpio::gpiod::PD9<gpio::Analog>,
    pub D15: gpio::gpiod::PD10<gpio::Analog>,
    pub D16: gpio::gpioh::PH8<gpio::Analog>,
    pub D17: gpio::gpioh::PH9<gpio::Analog>,
    pub D18: gpio::gpioh::PH10<gpio::Analog>,
    pub D19: gpio::gpioh::PH11<gpio::Analog>,
    pub D20: gpio::gpioh::PH12<gpio::Analog>,
    pub D21: gpio::gpioh::PH13<gpio::Analog>,
    pub D22: gpio::gpioh::PH14<gpio::Analog>,
    pub D23: gpio::gpioh::PH15<gpio::Analog>,
    pub D24: gpio::gpioi::PI0<gpio::Analog>,
    pub D25: gpio::gpioi::PI1<gpio::Analog>,
    pub D26: gpio::gpioi::PI2<gpio::Analog>,
    pub D27: gpio::gpioi::PI3<gpio::Analog>,
    pub D28: gpio::gpioi::PI6<gpio::Analog>,
    pub D29: gpio::gpioi::PI7<gpio::Analog>,
    pub D30: gpio::gpioi::PI9<gpio::Analog>,
    pub D31: gpio::gpioi::PI10<gpio::Analog>,
    // Control signals
    pub NBL0: gpio::gpioe::PE0<gpio::Analog>,    // Byte lane 0
    pub NBL1: gpio::gpioe::PE1<gpio::Analog>,    // Byte lane 1
    pub NBL2: gpio::gpioi::PI4<gpio::Analog>,    // Byte lane 2
    pub NBL3: gpio::gpioi::PI5<gpio::Analog>,    // Byte lane 3
    pub SDCKE0: gpio::gpioh::PH2<gpio::Analog>,  // Clock enable
    pub SDCLK: gpio::gpiog::PG8<gpio::Analog>,   // SDRAM clock
    pub SDNCAS: gpio::gpiog::PG15<gpio::Analog>, // Column address strobe
    pub SDNE0: gpio::gpioh::PH3<gpio::Analog>,   // Chip select
    pub SDRAS: gpio::gpiof::PF11<gpio::Analog>,  // Row address strobe
    pub SDNWE: gpio::gpioh::PH5<gpio::Analog>,   // Write enable
}

/// USB2 High-Speed pins (internal use only).
#[allow(non_snake_case, dead_code)]
pub(crate) struct Usb2Pins {
    pub DN: gpio::gpioa::PA11<gpio::Analog>, // USB2 D-
    pub DP: gpio::gpioa::PA12<gpio::Analog>, // USB2 D+
}
