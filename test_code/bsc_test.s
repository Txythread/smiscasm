.global main
.section "CODE"
main:
	adrp x0, msg@PAGE
	add x0, msg@PAGEOFF
	hlt

.section "DATA"
msg:
	.ascii "Hello, world!"
