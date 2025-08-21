**`jmpz` - Jump to value if another value is zero**

The `jmpz` instruction jumps to the second register specified or the immediate value if the value of the first argument (a register) is zero.  
It's recommended to **always jump to a register**, as the section might not start at 0 in the memory, in which case the behaviour is undefined.  
Always use `adrp` and `add` on another register to get the memory address.