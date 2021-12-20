# Error test cases

These are test cases that should yield errors, but I haven't started testing for any of that yet.

#### Operand out of range

```
LDR R4, R2, #-35 ; Out of range, LDR only supports [-32, 31]
```