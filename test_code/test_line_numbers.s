.section "CODE"

main:
	# line 5 should fail
	sdt x0, x4

!include debuglib

sdt 94

.section "DATA"
	sdt 94

.stc "Hello, world how are you asdfasdfasdf
