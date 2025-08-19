.global main
.section "CODE"
main:
	adrp x2, mul@PAGE
	add x2, mul@PAGEOFF
	adrp x3, op1@PAGE
	add x3, op1@PAGEOFF
	adrp x4, op2@PAGE
	add x4, op2@PAGEOFF
	#ble x2
	hlt

!include bscmath.s

.section "DATA"
op1:
	.ascii "a" # 65
op2:
	.ascii "b" # 66
