.ORIG x3000

LD R1, STOP_AT
ADD R2, R2, #5

LOOP
    JSR IS_DIVISIBLE
    ADD R1, R1, #-1
    BRp LOOP

    HALT

; R1 dividend
; R2 divisor
; R3 result (0/1)
IS_DIVISIBLE
    ST R1, R1_SAVE
    ST R2, R2_SAVE

    ; Flip sign of divisor
    ADD R2, R2, #-1
    NOT R2, R2

IS_DIVISIBLE_LOOP
    ADD R1, R1, R2
    BRp IS_DIVISIBLE_LOOP

    ; If we exit the loop with a negative
    ; value, the divident is not evenly 
    ; divisble by the divisor
    BRz IS_DIVISIBLE_YES

    AND R3, R3, #0
    LD R1, R1_SAVE
    LD R2, R2_SAVE

    RET

IS_DIVISIBLE_YES
    AND R3, R3, #0
    ADD R3, R3, #1
    LD R1, R1_SAVE
    LD R2, R2_SAVE
    RET


HALT
    
STOP_AT .FILL #32000

R1_SAVE .FILL x0000
R2_SAVE .FILL x0000
R3_SAVE .FILL x0000

SUB_ONE_MASK .FILL xFFFE

HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END