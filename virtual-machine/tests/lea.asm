.ORIG x3000
LEA R0, HELLO_STR                  ; 0x3000
HALT                               ; 0x3001
HELLO_STR .STRINGZ "Hello World!"  ; 0x3002
.END 