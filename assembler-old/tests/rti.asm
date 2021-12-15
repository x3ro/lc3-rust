.ORIG x3000
    ADD R0, R0, #7
    RTI
    HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END