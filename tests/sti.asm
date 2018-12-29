.ORIG x3000
    ADD R0, R0, #-8         ; 0x3000
    STI R0, SOME_ADDR       ; 0x3001
    HALT                    ; 0x3002

    SOME_VALUE  .FILL x0000 ; 0x3003
    SOME_ADDR   .FILL x3003 ; 0x3004
    HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END