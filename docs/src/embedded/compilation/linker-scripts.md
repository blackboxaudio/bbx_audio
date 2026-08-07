# Linker Scripts

Linker scripts tell the linker where to place code and data in memory. For embedded systems, this is critical—the microcontroller expects the vector table at a specific address, and DMA requires buffers in specific memory regions.

## What Linker Scripts Do

The linker combines compiled object files into a single executable. The linker script controls:

1. **Memory regions**: Define available RAM and flash with addresses and sizes
2. **Section placement**: Where `.text`, `.data`, `.bss` go
3. **Symbol definitions**: `_stack_start`, `_heap_size`, etc.
4. **Entry point**: Where execution begins after reset

Without a linker script, the linker doesn't know your chip's memory layout and produces an unusable binary.

## The Two-Script System

Embedded Rust uses two linker scripts:

```
memory.x     (You provide)      Defines memory regions
     ↓
  link.x     (cortex-m-rt)      Defines sections and startup
```

**`memory.x`**: You write this. Describes your specific chip's memory.

**`link.x`**: Provided by `cortex-m-rt`. Uses your `memory.x` definitions to place standard sections.

## memory.x Structure

Here's a complete `memory.x` for the Daisy/STM32H750:

```ld
/* memory.x - STM32H750 memory layout */

MEMORY
{
    /* Internal Flash - 128KB
       Boot code and vector table must start here */
    FLASH  : ORIGIN = 0x08000000, LENGTH = 128K

    /* DTCM - 128KB, tightly coupled, fastest
       Cannot be accessed by DMA
       Use for stack and performance-critical data */
    DTCM   : ORIGIN = 0x20000000, LENGTH = 128K

    /* AXI SRAM - 512KB, fast
       D1 domain, DMA accessible via MDMA */
    AXI    : ORIGIN = 0x24000000, LENGTH = 512K

    /* SRAM1 - 128KB
       D2 domain, DMA1/DMA2 accessible */
    SRAM1  : ORIGIN = 0x30000000, LENGTH = 128K

    /* SRAM2 - 128KB
       D2 domain, DMA1/DMA2 accessible */
    SRAM2  : ORIGIN = 0x30020000, LENGTH = 128K

    /* SRAM3 - 32KB
       D2 domain, DMA1/DMA2 accessible
       Ideal for audio DMA buffers */
    SRAM3  : ORIGIN = 0x30040000, LENGTH = 32K

    /* SRAM4 - 64KB
       D3 domain, BDMA accessible */
    SRAM4  : ORIGIN = 0x38000000, LENGTH = 64K

    /* External QSPI Flash - 8MB
       Execute-in-place possible but slower */
    QSPI   : ORIGIN = 0x90000000, LENGTH = 8M

    /* External SDRAM - 64MB
       Connected via FMC, slowest */
    SDRAM  : ORIGIN = 0xC0000000, LENGTH = 64M
}

/* Assign standard sections to regions */
REGION_ALIAS("REGION_TEXT", FLASH);
REGION_ALIAS("REGION_RODATA", FLASH);
REGION_ALIAS("REGION_DATA", DTCM);
REGION_ALIAS("REGION_BSS", DTCM);
REGION_ALIAS("REGION_STACK", DTCM);
```

### MEMORY Block

Each line in `MEMORY` defines a region:

```ld
NAME : ORIGIN = address, LENGTH = size
```

- **ORIGIN**: Physical start address (from datasheet)
- **LENGTH**: Size in bytes (K = 1024, M = 1048576)

### REGION_ALIAS

These aliases tell `link.x` (from cortex-m-rt) where to place standard sections:

| Alias | Contents | Typical Region |
|-------|----------|----------------|
| `REGION_TEXT` | Executable code | FLASH |
| `REGION_RODATA` | Constants, strings | FLASH |
| `REGION_DATA` | Initialized static variables | DTCM |
| `REGION_BSS` | Zero-initialized statics | DTCM |
| `REGION_STACK` | Stack space | DTCM |

## Custom Sections for DMA

Audio DMA buffers must be in DMA-accessible memory. Define custom sections:

```ld
/* Add to memory.x */

SECTIONS
{
    /* DMA-accessible audio buffers in SRAM3 */
    .sram3 (NOLOAD) : ALIGN(4)
    {
        _sram3_start = .;
        *(.sram3 .sram3.*);
        . = ALIGN(4);
        _sram3_end = .;
    } > SRAM3

    /* Large buffers in external SDRAM */
    .sdram (NOLOAD) : ALIGN(4)
    {
        _sdram_start = .;
        *(.sdram .sdram.*);
        . = ALIGN(4);
        _sdram_end = .;
    } > SDRAM
}
```

### Section Syntax Explained

```ld
.section_name (NOLOAD) : ALIGN(4)
{
    _section_start = .;     /* Symbol for start address */
    *(.section .section.*); /* Match all input sections */
    . = ALIGN(4);           /* Align to 4 bytes */
    _section_end = .;       /* Symbol for end address */
} > MEMORY_REGION
```

**`(NOLOAD)`**: Don't include in binary. The section exists in RAM but isn't loaded from flash. Use for buffers that will be initialized at runtime.

**`ALIGN(4)`**: Ensure 4-byte alignment. Required for ARM DMA.

**`*(.sram3 .sram3.*)`**: Match input sections named `.sram3` and `.sram3.anything`.

### Using Custom Sections in Rust

Place data in custom sections with the `link_section` attribute:

```rust
// Audio DMA buffers - must be in SRAM3 for DMA access
#[link_section = ".sram3"]
static mut RX_BUFFER: [f32; 256] = [0.0; 256];

#[link_section = ".sram3"]
static mut TX_BUFFER: [f32; 256] = [0.0; 256];

// Large delay buffer in external SDRAM
#[link_section = ".sdram"]
static mut DELAY_LINE: [f32; 2_000_000] = [0.0; 2_000_000];
```

## STM32H750 Memory Map

Here's the complete address space:

```
Address Range          Region              Notes
─────────────────────────────────────────────────────────
0x00000000-0x0001FFFF  ITCM               Instruction TCM (64KB)
0x08000000-0x0801FFFF  Flash Bank 1       Internal flash (128KB)
0x08100000-0x0811FFFF  Flash Bank 2       (if present)
0x20000000-0x2001FFFF  DTCM               Data TCM (128KB)
0x24000000-0x2407FFFF  AXI SRAM           D1 domain (512KB)
0x30000000-0x3001FFFF  SRAM1              D2 domain (128KB)
0x30020000-0x3003FFFF  SRAM2              D2 domain (128KB)
0x30040000-0x30047FFF  SRAM3              D2 domain (32KB)
0x38000000-0x3800FFFF  SRAM4              D3 domain (64KB)
0x40000000-0x5FFFFFFF  Peripherals        APB/AHB peripherals
0x90000000-0x9FFFFFFF  QSPI               External flash
0xC0000000-0xCFFFFFFF  FMC SDRAM          External SDRAM
```

## link.x Reference

The `link.x` from `cortex-m-rt` handles:

**Vector Table**: Placed at start of FLASH
```ld
.vector_table ORIGIN(FLASH) :
{
    KEEP(*(.vector_table.reset_vector));
    KEEP(*(.vector_table.exceptions));
}
```

**Initialization Data**: Source in flash, copied to RAM
```ld
.data : AT(__sidata)  /* Load address in flash */
{
    __sdata = .;
    *(.data .data.*);
    __edata = .;
} > REGION_DATA
```

**BSS Section**: Zeroed at startup
```ld
.bss (NOLOAD) :
{
    __sbss = .;
    *(.bss .bss.*);
    __ebss = .;
} > REGION_BSS
```

## Stack Configuration

The stack pointer is initialized at reset. Configure size in `memory.x`:

```ld
/* Stack size - adjust based on your needs */
_stack_size = 8K;

/* Stack placed at end of DTCM, growing downward */
_stack_start = ORIGIN(DTCM) + LENGTH(DTCM);
```

Or provide a `_stack_start` symbol directly:

```ld
_stack_start = 0x20020000;  /* End of DTCM */
```

The Cortex-M7 decrements the stack pointer before storing, so `_stack_start` is one past the last valid stack address.

## Common Linker Errors

### "memory region FLASH is full"

Your code exceeds flash capacity.

**Diagnose**:
```bash
arm-none-eabi-size target/thumbv7em-none-eabihf/release/app
```

**Fix**:
- Enable LTO
- Use `opt-level = "z"`
- Remove unused dependencies
- Move large data to QSPI or SDRAM

### "undefined symbol: _stack_start"

Missing `_stack_start` definition.

**Fix**: Add to `memory.x`:
```ld
_stack_start = ORIGIN(DTCM) + LENGTH(DTCM);
```

### "section `.bss' will not fit in region"

Static variables exceed RAM.

**Fix**:
- Move large buffers to SDRAM
- Reduce buffer sizes
- Use `#[link_section]` to spread data across regions

### "address 0xXXXXXXXX out of range"

Code references an address outside defined regions.

**Fix**: Check that all `ORIGIN` and `LENGTH` values match your chip's datasheet.

## Verifying the Memory Map

After building, inspect the binary:

```bash
# Show section sizes and addresses
arm-none-eabi-objdump -h target/thumbv7em-none-eabihf/release/app

# Show all symbols with addresses
arm-none-eabi-nm -n target/thumbv7em-none-eabihf/release/app

# Detailed section info
arm-none-eabi-readelf -S target/thumbv7em-none-eabihf/release/app
```

Example output:
```
Sections:
Idx Name          Size      VMA       LMA       Type
  0 .vector_table 00000400  08000000  08000000  DATA
  1 .text         0000b234  08000400  08000400  TEXT
  2 .rodata       00001a20  0800b634  0800b634  DATA
  3 .data         00000080  20000000  0800d054  DATA
  4 .bss          00001000  20000080  20000080  NOBITS
  5 .sram3        00000800  30040000  30040000  NOBITS
```

## Further Reading

- [Binary Formats](binary-formats.md) - ELF structure and flashing
- [Memory Model](../fundamentals/memory-model.md) - How Rust uses these regions
- [Toolchain](toolchain.md) - How the linker is invoked
