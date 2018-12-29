.ORIG x3000
    JSR INCREMENT_R0 ; 0x3000
    HALT             ; 0x3001
    NOP              ; 0x3002
    ADD R1, R1, #1   ; 0x3003 This should never be executed

INCREMENT_R0
    ADD R0, R0, #1   ; 0x3004
    RET              ; 0x3005

    HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END