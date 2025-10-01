.my_number 944418648 # 0xaf804358

.section "CODE"

_start:
	# Load my number into x0
	mov x0, my_number@MSB
	lshb x0
	add x0, my_number@B2
	lshb x0
	add x0, my_number@B1
	lshb x0
	add x0, my_number@LSB


	hlt
