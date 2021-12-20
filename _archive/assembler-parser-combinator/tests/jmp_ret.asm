.ORIG x3000
START
    ; R2 is not actually set here, so this test would not run properly
    JMP R2
    RET
    HALT
    HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END