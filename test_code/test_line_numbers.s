.section "CODE"

main:
	# line 5 should fail
	add x0, 4
	add x3, x3

!include debuglib

.x 1
.y 2
.asf [x + y]
.section "DATA"
	sub x0, 94

.stc "Hello, world how are you asdfasdfasdf"
