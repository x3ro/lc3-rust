.ORIG x3000
    LEA R0, SECOND  ; 0x3000
    LDR R1, R0, #-1 ; 0x3001 - This should now be 0xFFFF
    HALT            ; 0x3002

    FIRST       .FILL xFFFF ; 0x3003
    SECOND      .FILL 42    ; 0x3004
    HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END