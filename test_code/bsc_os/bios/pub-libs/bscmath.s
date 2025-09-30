# Multiply x0 by x1 and store the result in x0
# x17, x18 and x19 will be changed
mul:
	adrp x19, bscmath_mul_end@PAGE
	add x19, bscmath_mul_end@PAGEOFF
	adrp x18, bscmath_mul_loop@PAGE
	add x18, bscmath_mul_loop@PAGEOFF
	mov x17, x0
	mov x0, 0
bscmath_mul_loop:
    jmpz x1, x19
    add x0, x17
    sub x1, 1
	jmp x18
bscmath_mul_end:
	ret
