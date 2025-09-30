.section "CODE"

_start:
	# Move 0x30_00_00_00 (disk start) into x5
	
	mov x5, 10

	# And 0x0 into x6
	mov x6, 0x0
	
	

load_loop:
	# Load the address
	lb x0, x5
	# And store it into ram
	sb x0, x6

	add x5, 0x1
	add x6, 0x1
	jmp load_loop@PAGEOFF
	



!include bscmath
