.section "CODE"

main:
	# line 5 should fail
	sdt x0, x4
	add x3, x33

!include debuglib
.
sdt 94

.x 1
.y 2
.asf [x + y]
.section "DATA"
	sdt 94

.stc "Hello, world how are you asdfasdfasdf"
