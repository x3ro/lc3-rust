.ORIG xBEEF
START
    ADD R0, R0, #7
    STI R0, TARGET
    TARGET .FILL x0000
    HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END