.ORIG x3000
    LEA R0, SECOND  ; 0x3000
    ADD R1, R1, #-9 ; 0x3001
    STR R1, R0, #-1 ; 0x3002
    HALT            ; 0x3003

    FIRST       .FILL x0000 ; 0x3004
    SECOND      .FILL 42    ; 0x3005
    HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END