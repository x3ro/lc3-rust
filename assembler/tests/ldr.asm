.ORIG x1234
START
    LDR R4, R2, #-5
    HALT
HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END