.ORIG x1234
START
    LD R0, SUB
    JSRR R0
    HALT
SUB
    ADD R0, R0, #1
    RET
HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END