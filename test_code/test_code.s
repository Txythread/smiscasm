.global main
.section "CODE"
main:
	adrp x0, msg@PAGE
	add x0, msg@PAGEOFF
	adrp x1, msg_end@PAGE
	add x1, msg_end@PAGEOFF
	sub x1, 1
	
loop:
	lb x2, x0
	add x0, 1
	out x2
	mov x0, x3
	sub x3, x1
	#jmpz x3, end
	#jmp loop

end:
	hlt

.section "DATA"
msg:
	.ascii "Hello, world!"
msg_end:
