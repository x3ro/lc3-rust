.ORIG x3000
    ; R0 is the register to trigger the right condition codes
    ; R1 holds the number of failed branches, i.e. non of the ADDs to R1 should ever be executed
    ; R2 holds the number of correct branches, to make sure we executed the entire file

    ; Positive branch
    LD R0, ZERO
    ADD R0, R0, #1 ; = 1, Positive condition should be set
    BRp SKIP_P
    ADD R1, R1, #1 ; This should be skipped
SKIP_P
    ADD R2, R2, #1 ; 1
    HALT

    ; Zero branch
    LD R0, ONE
    ADD R0, R0, #-1 ; = 0, zero condition should be set
    BRz SKIP_Z
    ADD R1, R1, #1 ; This should be skipped
SKIP_Z
    ADD R2, R2, #1 ; 2
    HALT

    ; Negative branch
    LD R0, ZERO
    ADD R0, R0, #-1 ; = -1, negative condition should be set
    BRn SKIP_N
    ADD R1, R1, #1 ; This should be skipped
SKIP_N
    ADD R2, R2, #1 ; 3
    HALT

    ; Positive or zero branch (zero)
    LD R0, ONE
    ADD R0, R0, #-1 ; = 0, zero condition should be set
    BRzp SKIP_ZP1
    ADD R1, R1, #1 ; This should be skipped
SKIP_ZP1
    ADD R2, R2, #1 ; 4
    HALT

    ; Positive or zero branch (positive)
    LD R0, ZERO
    ADD R0, R0, #1 ; = 1, positive condition should be set
    BRzp SKIP_ZP2
    ADD R1, R1, #1 ; This should be skipped
SKIP_ZP2
    ADD R2, R2, #1 ; 5
    HALT

    ; Negative or zero branch (zero)
    LD R0, ONE
    ADD R0, R0, #-1
    BRnz SKIP_NZ1
    ADD R1, R1, #1 ; This should be skipped
SKIP_NZ1
    ADD R2, R2, #1 ; 6
    HALT

    ; Negative or zero branch (negative)
    LD R0, ZERO
    ADD R0, R0, #-1
    BRnz SKIP_NZ2
    ADD R1, R1, #1 ; This should be skipped
SKIP_NZ2
    ADD R2, R2, #1 ; 7
    HALT

    ; Negative or positive branch (negative)
    LD R0, ZERO
    ADD R0, R0, #-1
    BRnp SKIP_NP1
    ADD R1, R1, #1 ; This should be skipped
SKIP_NP1
    ADD R2, R2, #1 ; 8
    HALT

    ; Negative or positive branch (positive)
    LD R0, ZERO
    ADD R0, R0, #1
    BRnp SKIP_NP2
    ADD R1, R1, #1 ; This should be skipped
SKIP_NP2
    ADD R2, R2, #1 ; 9
    HALT

    ; Unconditional branch (zero)
    LD R0, ONE
    ADD R0, R0, #-1
    BR SKIP_ZPN1
    ADD R1, R1, #1 ; This should be skipped
SKIP_ZPN1
    ADD R2, R2, #1 ; 10
    HALT

    ; Unconditional branch (positive)
    LD R0, ZERO
    ADD R0, R0, #1
    BR SKIP_ZPN2
    ADD R1, R1, #1 ; This should be skipped
SKIP_ZPN2
    ADD R2, R2, #1 ; 11
    HALT

    ; Unconditional branch (negative)
    LD R0, ZERO
    ADD R0, R0, #-1
    BR SKIP_ZPN3
    ADD R1, R1, #1 ; This should be skipped
SKIP_ZPN3
    ADD R2, R2, #1 ; 12
    HALT

ZERO .FILL x0000
ONE  .FILL x0001
HELLO_STR .STRINGZ "If I don't add this the assembler segfaults"
.END