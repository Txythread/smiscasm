**`lb` - Load Byte**

Loads a byte in memory at the given address (immediate or register) into the first argument (a register) specified.  
It's recommend to **always use a register for the address**, never an immediate value, as the section start might not be loaded into 0.  
Please always load the address into a register (via `adrp` and `add`) and pass that register as the second argument.