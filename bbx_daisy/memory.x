/* STM32H750VBTx memory layout for Daisy Seed */

MEMORY
{
    /* Flash memory - 128KB internal (Daisy uses external QSPI for larger programs) */
    FLASH (rx)  : ORIGIN = 0x08000000, LENGTH = 128K

    /* DTCM (Data Tightly Coupled Memory) - 128KB, fastest access for stack/heap */
    DTCM (rwx)  : ORIGIN = 0x20000000, LENGTH = 128K

    /* AXI SRAM - 512KB, general purpose RAM (D1 domain) */
    RAM (rwx)   : ORIGIN = 0x24000000, LENGTH = 512K

    /* SRAM1 - 128KB (D2 domain, accessible by DMA1/DMA2) */
    SRAM1 (rwx) : ORIGIN = 0x30000000, LENGTH = 128K

    /* SRAM2 - 128KB (D2 domain, accessible by DMA1/DMA2) */
    SRAM2 (rwx) : ORIGIN = 0x30020000, LENGTH = 128K

    /* SRAM3 - 32KB (D2 domain, ideal for DMA buffers) */
    SRAM3 (rwx) : ORIGIN = 0x30040000, LENGTH = 32K

    /* SRAM4 - 64KB (D3 domain, battery-backed) */
    SRAM4 (rwx) : ORIGIN = 0x38000000, LENGTH = 64K

    /* Backup SRAM - 4KB (battery-backed, persistent storage) */
    BACKUP (rw) : ORIGIN = 0x38800000, LENGTH = 4K
}

/* Stack configuration */
_stack_start = ORIGIN(DTCM) + LENGTH(DTCM);

/* DMA buffer placement - must be in SRAM accessible by DMA */
SECTIONS
{
    /* Audio DMA buffers go in SRAM3 (D2 domain, DMA-accessible) */
    .sram3 (NOLOAD) : ALIGN(4)
    {
        *(.sram3 .sram3.*);
        . = ALIGN(4);
    } > SRAM3

    /* Large buffers can go in SRAM1/SRAM2 */
    .sram1 (NOLOAD) : ALIGN(4)
    {
        *(.sram1 .sram1.*);
        . = ALIGN(4);
    } > SRAM1

    /* Persistent data in battery-backed SRAM */
    .backup (NOLOAD) : ALIGN(4)
    {
        *(.backup .backup.*);
        . = ALIGN(4);
    } > BACKUP
}
