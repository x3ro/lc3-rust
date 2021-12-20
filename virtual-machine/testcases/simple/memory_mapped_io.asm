.ORIG x3000
    ; Store something in memory, which will trigger the
    ; test memory-mapped I/O device to write something
    ADD R0, R0, #1
    STI R0, WRITE_ADDR

WAIT_FOR_IO
    LDI R0, READ_ADDR
    BRnz WAIT_FOR_IO
    HALT

    READ_ADDR  .FILL xFE00
    WRITE_ADDR .FILL xFE01 ; Address we're going to write to in
                           ; a separate thread of our VM, simulating
                           ; memory mapped I/O

    HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END