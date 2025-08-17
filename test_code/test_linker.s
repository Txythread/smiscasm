.section "CODE"
main:
	adrp x1, msg1@PAGE
	add x1, msg1@PAGEOFF
	out x1
	adrp x2, msg2@PAGE
	add x2, msg2@PAGEOFF
	out x2
	adrp x3, code@PAGE
	add x3, code@PAGEOFF
	jmp x3
msg1:
	.ascii "H"
.section "CODE2"
code:
	adrp x2, main@PAGE
	add x2, main@PAGEOFF
	jmp main@PAGE
.section "DATA"
msg2:
	.ascii "W"
