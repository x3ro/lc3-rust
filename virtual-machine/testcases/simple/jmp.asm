.ORIG x3000
    LD R1, ADDR      ; 0x3000
    JMP R1           ; 0x3001
    ADD R0, R0, #5   ; 0x3002 - should be skipped
    ADD R0, R0, #1   ; 0x3003
    HALT             ; 0x3004

    ADDR        .FILL x3003
    HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END