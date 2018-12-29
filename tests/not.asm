.ORIG x3000
    LD R0, SOME_VALUE
    NOT R1, R0
    HALT

    SOME_VALUE  .FILL x1234 ; NOT(0x1234) = 0xEDCB
    HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END