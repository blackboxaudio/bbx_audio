//! Early hardware initialization.
//!
//! This module provides the `pre_init` function that cortex-m-rt calls
//! before main() to configure critical hardware features.

use cortex_m::interrupt;

/// Pre-initialization function called by cortex-m-rt before main().
///
/// This function is called immediately after the Reset handler initializes
/// .data and .bss sections, but before main() is called.
///
/// On STM32H7, this function performs critical early initialization:
/// - Resets the RCC (Reset and Clock Control) to a known state
/// - Enables the FPU (Floating Point Unit)
///
/// This matches the behavior of the C++ SystemInit function from ST's CMSIS.
///
/// # Safety
///
/// This function is unsafe because it directly accesses hardware registers.
/// It is automatically called by cortex-m-rt's startup code and should never
/// be called manually.
#[cortex_m_rt::pre_init]
unsafe fn pre_init() {
    // STM32H7 RCC register addresses
    const RCC_BASE: u32 = 0x5802_4400;
    const RCC_CR: *mut u32 = RCC_BASE as *mut u32;
    const RCC_CFGR: *mut u32 = (RCC_BASE + 0x10) as *mut u32;
    const RCC_D1CFGR: *mut u32 = (RCC_BASE + 0x18) as *mut u32;
    const RCC_D2CFGR: *mut u32 = (RCC_BASE + 0x1C) as *mut u32;
    const RCC_D3CFGR: *mut u32 = (RCC_BASE + 0x20) as *mut u32;
    const RCC_PLLCKSELR: *mut u32 = (RCC_BASE + 0x28) as *mut u32;
    const RCC_PLLCFGR: *mut u32 = (RCC_BASE + 0x2C) as *mut u32;
    const RCC_PLL1DIVR: *mut u32 = (RCC_BASE + 0x30) as *mut u32;
    const RCC_PLL1FRACR: *mut u32 = (RCC_BASE + 0x34) as *mut u32;
    const RCC_PLL2DIVR: *mut u32 = (RCC_BASE + 0x38) as *mut u32;
    const RCC_PLL2FRACR: *mut u32 = (RCC_BASE + 0x3C) as *mut u32;
    const RCC_PLL3DIVR: *mut u32 = (RCC_BASE + 0x40) as *mut u32;
    const RCC_PLL3FRACR: *mut u32 = (RCC_BASE + 0x44) as *mut u32;
    const RCC_CIER: *mut u32 = (RCC_BASE + 0x60) as *mut u32;
    const RCC_AHB2ENR: *mut u32 = (RCC_BASE + 0xDC) as *mut u32;

    // DBGMCU register for chip revision detection
    const DBGMCU_IDCODE: *const u32 = 0x5C00_1000 as *const u32;

    // AXI SRAM switch matrix register (for revision Y workaround)
    const AXI_TARG7_FN_MOD_ISS_BM: *mut u32 = 0x5100_8108 as *mut u32;

    // ARM Cortex-M7 System Control Block registers
    const SCB_VTOR: *mut u32 = 0xE000_ED08 as *mut u32; // Vector Table Offset Register
    const CPACR: *mut u32 = 0xE000_ED88 as *mut u32; // Coprocessor Access Control Register

    // Vector table and FPU configuration
    const FLASH_BASE: u32 = 0x0800_0000; // Flash base address for STM32H7
    const FPU_ENABLE: u32 = (3 << 20) | (3 << 22); // CP10 and CP11 = 0b11 (full access)

    interrupt::free(|_| {
        // SAFETY: All register addresses are valid for STM32H7
        // We're in pre_init, so no other code is running

        // Reset the RCC clock configuration to default state
        // This matches SystemInit() from ST's system_stm32h7xx.c

        // Set HSION bit (enable HSI oscillator)
        let rcc_cr = unsafe { RCC_CR.read_volatile() };
        unsafe { RCC_CR.write_volatile(rcc_cr | (1 << 0)) }; // RCC_CR_HSION

        // Wait for HSI to be ready (HSIRDY bit, bit 2)
        // Timeout after ~1000 iterations to avoid infinite loop
        let mut timeout = 1000u32;
        while timeout > 0 {
            let rcc_cr = unsafe { RCC_CR.read_volatile() };
            if (rcc_cr & (1 << 2)) != 0 {
                // HSIRDY is set, HSI is ready
                break;
            }
            timeout -= 1;
        }

        // Reset CFGR register
        unsafe { RCC_CFGR.write_volatile(0x0000_0000) };

        // Reset HSEON, CSSON, CSION, RC48ON, CSIKERON, PLL1ON, PLL2ON, PLL3ON bits
        let rcc_cr = unsafe { RCC_CR.read_volatile() };
        unsafe { RCC_CR.write_volatile(rcc_cr & 0xEAF6_ED7F) };

        // Reset domain configuration registers
        unsafe { RCC_D1CFGR.write_volatile(0x0000_0000) };
        unsafe { RCC_D2CFGR.write_volatile(0x0000_0000) };
        unsafe { RCC_D3CFGR.write_volatile(0x0000_0000) };

        // Reset PLL configuration registers
        unsafe { RCC_PLLCKSELR.write_volatile(0x0000_0000) };
        unsafe { RCC_PLLCFGR.write_volatile(0x0000_0000) };
        unsafe { RCC_PLL1DIVR.write_volatile(0x0000_0000) };
        unsafe { RCC_PLL1FRACR.write_volatile(0x0000_0000) };
        unsafe { RCC_PLL2DIVR.write_volatile(0x0000_0000) };
        unsafe { RCC_PLL2FRACR.write_volatile(0x0000_0000) };
        unsafe { RCC_PLL3DIVR.write_volatile(0x0000_0000) };
        unsafe { RCC_PLL3FRACR.write_volatile(0x0000_0000) };

        // Reset HSEBYP bit
        let rcc_cr = unsafe { RCC_CR.read_volatile() };
        unsafe { RCC_CR.write_volatile(rcc_cr & 0xFFFB_FFFF) };

        // Disable all RCC interrupts
        unsafe { RCC_CIER.write_volatile(0x0000_0000) };

        // Enable D2 domain SRAM clocks
        // Required for AHB SRAM (SRAM1, SRAM2, SRAM3) in D2 domain
        // Bits 31, 30, 29 = SRAM3EN, SRAM2EN, SRAM1EN
        let rcc_ahb2enr = unsafe { RCC_AHB2ENR.read_volatile() };
        unsafe { RCC_AHB2ENR.write_volatile(rcc_ahb2enr | 0xE000_0000) };

        // Dummy read to ensure write completes
        let _ = unsafe { RCC_AHB2ENR.read_volatile() };

        // Check chip revision and apply AXI SRAM workaround if needed
        // STM32H7 revision Y (REV_ID < 0x2000) needs AXI SRAM switch matrix fix
        // REV_ID is in bits 15:0, DEV_ID is in bits 31:16
        let idcode = unsafe { DBGMCU_IDCODE.read_volatile() };
        if (idcode & 0x0000_FFFF) < 0x2000 {
            // Change the switch matrix read issuing capability to 1 for AXI SRAM target (Target 7)
            // This fixes a silicon errata on early STM32H7 revisions
            unsafe { AXI_TARG7_FN_MOD_ISS_BM.write_volatile(0x0000_0001) };
        }

        // Configure Vector Table Offset Register (VTOR)
        // Point to flash base address where our vector table is located
        unsafe { SCB_VTOR.write_volatile(FLASH_BASE) };

        // Enable FPU
        let cpacr = unsafe { CPACR.read_volatile() };
        unsafe { CPACR.write_volatile(cpacr | FPU_ENABLE) };
    });

    // Memory barriers to ensure all writes complete before continuing
    cortex_m::asm::dsb();
    cortex_m::asm::isb();
}
