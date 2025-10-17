.mode text
.section "CODE"


_start:
	mov x0, 1


!include debuglib
!include bscmath

.mode data
.stc "Hello, world!"
