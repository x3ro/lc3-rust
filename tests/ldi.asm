.ORIG x3000
    LDI R0, ADDR            ; 0x3000
    HALT                    ; 0x3001 - R0 should be mem[mem[ADDR]] = mem[0x3002] = 0xFFFF
    RESULT      .FILL xFFFF ; 0x3002
    ADDR        .FILL x3002 ; 0x3003
    HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END