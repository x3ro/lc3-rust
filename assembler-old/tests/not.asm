.ORIG x3000
    ADD R0, R0, #7
    NOT R4, R2
    HALT
HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END
