**`cal` - Jump with stored pc**

Sets the program counter to the second value specified (by a register) and saves the return value (on the stack).  
If the sp didn't change (or was restored), you can return using `ret`.

Please always load addresses using adrp (for @PAGE) & add (for @PAGEOFF).