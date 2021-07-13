.ORIG x3000
    ADD R0, R0, #7
    ADD R1, R1, #7
    ADD R2, R1, R2
    HALT

SOME_STRING
    .STRINGZ "foobarlol1"
    LD R0, SOME_STRING
.END
