.ORIG x3000                          ; this is the address in memory where the program will be loaded
LEA R0, HELLO_STR                    ; 0x3000 load the address of the HELLO_STR string into R0
PUTs                                 ; 0x3001 output the string pointed to by R0 to the console
HALT                                 ; 0x3002 halt the program
HELLO_STR .STRINGZ "Hello World!\n"  ; 0x3003 - ...... store this string here in the program
.END