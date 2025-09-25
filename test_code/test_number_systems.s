.section "CODE"

.hex-ten 0x10
.bin-ten 0b10
.octal-ten 0o10

main:
	add x0, 10		# + 10, 10
	add x0, hex-ten		# + 16, 26
	add x0, bin-ten		# +  2, 28
	add x0, octal-ten	# +  8, 36
	
	hlt
