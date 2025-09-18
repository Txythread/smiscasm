# Prints a string (via standard output (/standard transmitter))
# Inputs:
# x0: start position of string in memory
# x1: the length of the string
# Outputs:
# N/A
# Modified Registers:
# x0, x1, x17, x18, x19
print:
	# Register Mapping:
	# x0  ... position of the next character in memory
	# x17 ... print loop address
	# x18 ... print end address
	# x19 ... current character
	adrp x17, debuglib-print-loop@PAGE
	mov x18, x17 # The page will always be the same
	add x17, debuglib-print-loop@PAGEOFF
	add x18, debuglib-print-end@PAGEOFF

# Print character, increment, stop logic
debuglib-print-loop:
	lb x19, x0
	out x19
	add x0, 1
	sub x1, 1
	jmpz x1, x18
	jmp x17

# Return to previous function
debuglib-print-end:
	ret
