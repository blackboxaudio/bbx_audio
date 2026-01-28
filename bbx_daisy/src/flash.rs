//! High-level QSPI flash storage driver for IS25LP064.
//!
//! Provides access to the 8MB on-board flash storage with simple read/write/erase operations.
//! Internal QSPI details (commands, sectors, QPI mode) are hidden from users.
//!
//! # Example
//!
//! ```ignore
//! // Flash is initialized via Board
//! let mut flash = board.flash();
//!
//! // Write some data
//! let data = [1u8, 2, 3, 4];
//! flash.write(0x1000, &data);
//!
//! // Read it back
//! let mut buffer = [0u8; 4];
//! flash.read(0x1000, &mut buffer);
//! ```

use crate::{
    hal,
    hal::{
        gpio::Speed,
        prelude::*,
        xspi::{Config, Qspi, QspiMode, QspiWord},
    },
    pins::QspiFlashPins,
};

// IS25LP064 command set (from datasheet)
#[allow(dead_code)]
const WRITE_STATUS_REGISTRY_CMD: u8 = 0x01; // WRSR
#[allow(dead_code)]
const WRITE_CMD: u8 = 0x02; // PP (Page Program)
#[allow(dead_code)]
const READ_STATUS_REGISTRY_CMD: u8 = 0x05; // RDSR
#[allow(dead_code)]
const WRITE_ENABLE_CMD: u8 = 0x06; // WREN
#[allow(dead_code)]
const ENTER_QPI_MODE_CMD: u8 = 0x35; // QPIEN
#[allow(dead_code)]
const SET_READ_PARAMETERS_CMD: u8 = 0xC0; // SRP
#[allow(dead_code)]
const SECTOR_ERASE_CMD: u8 = 0xD7; // SER (4KB sector erase)
#[allow(dead_code)]
const FAST_READ_QUAD_IO_CMD: u8 = 0xEB; // FRQIO

// Memory specifications
const SECTOR_SIZE: u32 = 4096; // 4KB sectors
const PAGE_SIZE: u32 = 256; // 256-byte pages
const CAPACITY: usize = 8 * 1024 * 1024; // 8MB
const MAX_ADDRESS: u32 = 0x7FFFFF; // 8MB - 1

/// On-board QSPI flash storage (8MB IS25LP064).
///
/// Provides high-level read/write/erase operations for the flash memory.
/// All internal details (QSPI commands, QPI mode, sector/page handling) are hidden.
#[allow(dead_code)]
pub struct Flash {
    driver: Qspi<hal::pac::QUADSPI>,
}

#[allow(dead_code)]
impl Flash {
    /// Initialize the flash driver.
    ///
    /// This is called internally by Board initialization. Users should access
    /// flash via `board.flash()` rather than calling this directly.
    pub(crate) fn new(
        clocks: &hal::rcc::CoreClocks,
        qspi_device: hal::pac::QUADSPI,
        qspi_peripheral: hal::rcc::rec::Qspi,
        pins: QspiFlashPins,
    ) -> Self {
        // Configure pins for high-speed QSPI operation
        // CS pin must be acquired and configured even though not directly used
        let mut cs = pins.CS.into_alternate::<10>();
        let mut sck = pins.SCK.into_alternate::<9>();
        let mut io0 = pins.IO0.into_alternate::<10>();
        let mut io1 = pins.IO1.into_alternate::<10>();
        let mut io2 = pins.IO2.into_alternate::<9>();
        let mut io3 = pins.IO3.into_alternate::<9>();

        cs.set_speed(Speed::VeryHigh);
        sck.set_speed(Speed::VeryHigh);
        io0.set_speed(Speed::VeryHigh);
        io1.set_speed(Speed::VeryHigh);
        io2.set_speed(Speed::VeryHigh);
        io3.set_speed(Speed::VeryHigh);

        // Initialize QSPI in single-bit mode first
        let qspi = qspi_device.bank1(
            (sck, io0, io1, io2, io3),
            Config::new(133.MHz()).mode(QspiMode::OneBit),
            clocks,
            qspi_peripheral,
        );

        let mut flash = Self { driver: qspi };

        // Configure the flash chip for optimal operation
        flash.enable_qpi_mode();
        flash.reset_status_register();
        flash.reset_read_register();

        flash
    }

    /// Get the flash capacity in bytes.
    #[inline]
    pub fn capacity(&self) -> usize {
        CAPACITY
    }

    /// Get the sector size in bytes.
    #[inline]
    pub fn sector_size(&self) -> usize {
        SECTOR_SIZE as usize
    }

    /// Read data from flash into the provided buffer.
    ///
    /// Reads consecutive bytes starting at `address` to fill `buffer`.
    /// If the read reaches the end of flash, it wraps around to the beginning.
    ///
    /// # Panics
    ///
    /// Panics if `address` is outside the valid range (0 to 0x7FFFFF).
    pub fn read(&mut self, address: u32, buffer: &mut [u8]) {
        assert!(address <= MAX_ADDRESS, "address out of range");

        // Read in chunks of 32 bytes (limitation of read_extended)
        for (i, chunk) in buffer.chunks_mut(32).enumerate() {
            self.driver
                .read_extended(
                    QspiWord::U8(FAST_READ_QUAD_IO_CMD),
                    QspiWord::U24(address + i as u32 * 32),
                    QspiWord::U8(0x00),
                    8, // 8 dummy cycles for FRQIO
                    chunk,
                )
                .unwrap();
        }
    }

    /// Write data to flash at the specified address.
    ///
    /// This method automatically erases the affected sectors before writing.
    /// All sectors that will be written to are completely erased (4KB each).
    ///
    /// If the write reaches the end of flash, it wraps around to the beginning.
    ///
    /// # Panics
    ///
    /// Panics if `address` is outside the valid range (0 to 0x7FFFFF).
    /// Panics if `data` is empty.
    pub fn write(&mut self, mut address: u32, data: &[u8]) {
        assert!(address <= MAX_ADDRESS, "address out of range");
        assert!(!data.is_empty(), "data cannot be empty");

        // Erase all affected sectors first
        self.erase(address, data.len() as u32);

        let mut length = data.len() as u32;
        let mut start_cursor = 0;

        loop {
            // Calculate bytes remaining until end of current page
            let page_remainder = PAGE_SIZE - (address & (PAGE_SIZE - 1));

            // Write data to the page in chunks of 32 bytes (limitation of write_extended)
            let size = page_remainder.min(length) as usize;
            for (i, chunk) in data[start_cursor..start_cursor + size].chunks(32).enumerate() {
                self.enable_write();
                self.driver
                    .write_extended(
                        QspiWord::U8(WRITE_CMD),
                        QspiWord::U24(address + i as u32 * 32),
                        QspiWord::None,
                        chunk,
                    )
                    .unwrap();
                self.wait_for_write();
            }
            start_cursor += size;

            // Stop if this was the last needed page
            if length <= page_remainder {
                break;
            }
            length -= page_remainder;

            // Move to the next page (wrap around at end of memory)
            address += page_remainder;
            address %= MAX_ADDRESS;
        }
    }

    /// Erase flash sectors covering the specified range.
    ///
    /// Erases all sectors that overlap with the range [address, address + length).
    /// Even if only one byte falls within a sector, the entire 4KB sector is erased.
    ///
    /// If the range reaches the end of flash, it wraps around to the beginning.
    ///
    /// # Panics
    ///
    /// Panics if `address` is outside the valid range (0 to 0x7FFFFF).
    /// Panics if `length` is zero.
    pub fn erase(&mut self, mut address: u32, mut length: u32) {
        assert!(address <= MAX_ADDRESS, "address out of range");
        assert!(length > 0, "length cannot be zero");

        loop {
            // Erase the current sector
            self.enable_write();
            self.driver
                .write_extended(
                    QspiWord::U8(SECTOR_ERASE_CMD),
                    QspiWord::U24(address),
                    QspiWord::None,
                    &[],
                )
                .unwrap();
            self.wait_for_write();

            // Calculate bytes remaining until end of current sector
            let sector_remainder = SECTOR_SIZE - (address & (SECTOR_SIZE - 1));

            // Stop if this was the last affected sector
            if length <= sector_remainder {
                break;
            }
            length -= sector_remainder;

            // Move to the next sector (wrap around at end of memory)
            address += sector_remainder;
            address %= MAX_ADDRESS;
        }
    }

    /// Reset the status register to driver defaults.
    fn reset_status_register(&mut self) {
        self.enable_write();
        self.driver
            .write_extended(
                QspiWord::U8(WRITE_STATUS_REGISTRY_CMD),
                QspiWord::U8(0b0000_0010),
                QspiWord::None,
                &[],
            )
            .unwrap();
        self.wait_for_write();
    }

    /// Reset the read register to driver defaults.
    fn reset_read_register(&mut self) {
        self.enable_write();
        self.driver
            .write_extended(
                QspiWord::U8(SET_READ_PARAMETERS_CMD),
                QspiWord::U8(0b1111_1000),
                QspiWord::None,
                &[],
            )
            .unwrap();
        self.wait_for_write();
    }

    /// Enable QPI (Quad Peripheral Interface) mode for faster operations.
    fn enable_qpi_mode(&mut self) {
        self.enable_write();
        self.driver
            .write_extended(QspiWord::U8(ENTER_QPI_MODE_CMD), QspiWord::None, QspiWord::None, &[])
            .unwrap();

        // Switch QSPI peripheral to 4-bit mode
        self.driver.configure_mode(QspiMode::FourBit).unwrap();

        self.wait_for_write();
    }

    /// Enable write operations (required before each write/erase command).
    fn enable_write(&mut self) {
        self.driver
            .write_extended(QspiWord::U8(WRITE_ENABLE_CMD), QspiWord::None, QspiWord::None, &[])
            .unwrap();
    }

    /// Wait for the current write/erase operation to complete.
    fn wait_for_write(&mut self) {
        loop {
            let mut status: [u8; 1] = [0xFF; 1];
            self.driver
                .read_extended(
                    QspiWord::U8(READ_STATUS_REGISTRY_CMD),
                    QspiWord::None,
                    QspiWord::None,
                    0,
                    &mut status,
                )
                .unwrap();

            // WIP (Write In Progress) bit is bit 0
            if status[0] & 0x01 == 0 {
                break;
            }
        }
    }
}
