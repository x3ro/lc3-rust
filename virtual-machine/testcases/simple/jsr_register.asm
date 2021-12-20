.ORIG x3000
    LD R0, INCREMENT_R0 ; 0x3000
    JSRR R0             ; 0x3001
    HALT                ; 0x3002
    NOP                 ; 0x3003
    ADD R1, R1, #1      ; 0x3004 This should never be executed
    ; ---
    ADD R2, R2, #1      ; 0x3005
    RET                 ; 0x3006

    INCREMENT_R0 .FILL x3005
    HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END