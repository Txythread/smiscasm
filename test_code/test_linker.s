.section "CODE"
main:
	adrp x1, msg1@PAGE
	add x1, msg1@PAGEOFF
	adrp x2, msg2@PAGE
	add x2, msg2@PAGEOFF
msg1:
	.ascii "H"
.section "DATA"
msg2:
	.ascii "W"
