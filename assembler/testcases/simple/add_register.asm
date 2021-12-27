.ORIG x3000
LD R1, SOME_X
LD R2, SOME_Y
ADD R0, R0, R1 ; = 0 + 16 = 16
HALT
ADD R0, R0, R2 ; = 16 - 16 = 0
HALT
ADD R0, R0, R2 ;  = 0 - 16 = -16
HALT
SOME_X    .FILL x10   ;  16
SOME_Y    .FILL xFFF0 ; -16
HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END