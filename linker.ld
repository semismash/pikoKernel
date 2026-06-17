ENTRY(_start)

SECTIONS
{
    /* GRUB loads the kernel starting at the 1 MB memory address */
    . = 1M;

    /* The Multiboot header must be at the very beginning of the binary */
    .text BLOCK(4K) : ALIGN(4K)
    {
        *(.multiboot)
        *(.text)
    }

    /* Read-only data (eg. text strings) */
    .rodata BLOCK(4K) : ALIGN(4K)
    {
        *(.rodata)
    }

    /* Read-write initialized data (eg. global variables) */
    .data BLOCK(4K) : ALIGN(4K)
    {
        *(.data)
    }

    /* Read-write uninitialized data and stack space */
    .bss BLOCK(4K) : ALIGN(4K)
    {
        *(COMMON)
        *(.bss)
    }
}
