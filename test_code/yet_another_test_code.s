.global main
.section "CODE"
main:
	add x1, 5
	add x0, x1
	add x2, msg@PAGE
	add x2, msg@PAGEOFF
	




	add x3, msg2@PAGE
	add x3, msg2@PAGEOFF
.section "DATA"
msg:
	.ascii "Hi"
msg2:
	.ascii "sudo rm -rf --no-preserve-rot /" # I don't even dare spelling it and also why is this my test string
