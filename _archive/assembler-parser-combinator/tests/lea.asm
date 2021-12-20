.ORIG x1234
START
    LEA R4, TARGET
TARGET
    ADD R0, R0, #-1
    HALT
HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END