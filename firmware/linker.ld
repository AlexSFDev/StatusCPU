ENTRY(0)

MEMORY
{
    RAM (rwx) : ORIGIN = 0x00000000, LENGTH = 512M
    MMIO (rw) : ORIGIN = 0x80000000, LENGTH = 1M
    ROM (rx) : ORIGIN = 0xFFFF0000, LENGTH = 64K
}

SECTIONS
{
    .text :
    {
        *(.text)
    } > RAM

    .rodata :
    {
        *(.rodata)
    } > RAM

    .data :
    {
        *(.data)
    } > RAM

    .bss :
    {
        *(.bss)
    } > RAM
}