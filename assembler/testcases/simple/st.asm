.ORIG x3000
    ADD R0, R0, #-7         ; 0x3000
    ST R0, SOME_VALUE       ; 0x3001
    HALT                    ; 0x3002

    SOME_VALUE  .FILL x0000 ; 0x3003
    HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END