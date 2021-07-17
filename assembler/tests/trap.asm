.ORIG x5000
START
    TRAP x23
    GETC
    OUT
    PUTS
    IN
    PUTSP
    HALT
    HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END