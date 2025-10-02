.msb 0x40
.lsb 0x48

.mode text
.section "CODE"

_start:
	# Load msb lsb into x0
	mov x0, msb
	lshb x0
	add x0, lsb

	# Move by one bit more
	lsh x0

	
	hlt
