.ORIG x3000
START
    ADD R0, R0, #1
    ADD R1, R0, #-10
    BRn START
    HALT
    HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END