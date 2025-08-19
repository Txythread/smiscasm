# Multiply x0 by x1 and store the result in x0
# x17, x18 and x19 will be changed
mul:
	adrp x19, bscmath-mul-end@PAGE
	add x19, bscmath-mul-end@PAGEOFF
	adrp x18, bscmath-mul-loop@PAGE
	add x18, bscmath-mul-loop@PAGEOFF
	mov x17, x1
	mov x0, 0
bscmath-mul-loop:
        jmpz x1, x19
        add x0, x17
        sub x1, 1
	jmp x18
bscmath-mul-end:
	#ret
