.ORIG x3000
    LD R0, SOME_VALUE
    LD R1, OTHER_VALUE
    AND R2, R0, R1
    HALT

    AND R2, R0, #0
    HALT

    AND R2, R0, #15
    HALT

    SOME_VALUE  .FILL x120F
    OTHER_VALUE .FILL x1200
    HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END